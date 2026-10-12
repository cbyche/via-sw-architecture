import asyncio
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence
from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient


class ProviderTests(unittest.IsolatedAsyncioTestCase):
    def client(self, folder):
        evidence = Evidence(Path(folder))
        with patch('prototypes.candidates.dp41_45_poc.shared.providers.credential', return_value='TEST_ONLY'):
            client = ModelClient(evidence, max_cost_usd=1, budget_file=Path(folder)/'budget.json')
        return client, evidence

    @staticmethod
    def reply(text='{"ok":true}'):
        return {'id':'stub-provider-id','choices':[{'message':{'content':text}}],
                'usage':{'prompt_tokens':100,'completion_tokens':10}},200,{}

    async def test_local_cancel_retains_actual_paid_http_completion(self):
        with tempfile.TemporaryDirectory() as folder:
            client, evidence = self.client(folder)
            def delayed(body):
                time.sleep(.05)
                return self.reply()
            client._request = delayed
            job = asyncio.create_task(client.call([{'role':'user','content':'test'}],request_id='R'))
            await asyncio.sleep(.01)
            job.cancel()
            with self.assertRaises(asyncio.CancelledError):
                await job
            row = evidence.records['model-calls'][0]
            self.assertEqual(row['status'],'DISCARDED_AFTER_LOCAL_CANCEL')
            self.assertEqual(row['usage']['input_tokens'],100)
            self.assertGreater(row['cost_usd'],0)
            self.assertEqual(client.reserved,0)
            self.assertEqual(json.loads((Path(folder)/'budget.json').read_text())['calls'][row['call_id']],row['cost_usd'])
            evidence.close()

    async def test_invalid_encoding_retry_has_separate_usage_and_context(self):
        with tempfile.TemporaryDirectory() as folder:
            client, evidence = self.client(folder)
            replies = iter([self.reply('not json'),self.reply()])
            client._request = lambda body: next(replies)
            result = await client.call([{'role':'user','content':'test'}],request_id='R')
            self.assertTrue(result['ok'])
            rows = evidence.records['model-calls']
            self.assertEqual(len(rows),2)
            self.assertNotEqual(rows[0]['context']['sha256'],rows[1]['context']['sha256'])
            self.assertEqual(client.spent,sum(r['cost_usd'] for r in rows))
            evidence.close()

    async def test_missing_usage_is_provisioned_and_never_called_free(self):
        with tempfile.TemporaryDirectory() as folder:
            client, evidence = self.client(folder)
            response,status,headers = self.reply()
            response.pop('usage')
            client._request = lambda body: (response,status,headers)
            await client.call([{'role':'user','content':'test'}],request_id='R')
            row = evidence.records['model-calls'][0]
            self.assertIsNone(row['cost_usd'])
            self.assertEqual(row['usage_state'],'UNKNOWN')
            self.assertGreater(client.spent,0)
            evidence.close()

    async def test_global_cap_prevents_dispatch(self):
        with tempfile.TemporaryDirectory() as folder:
            client,evidence = self.client(folder)
            (Path(folder)/'budget.json').write_text(json.dumps({'provider':'qwen','limit_usd':0,'calls':{}}))
            client._request = lambda body: self.fail('must not dispatch')
            with self.assertRaisesRegex(RuntimeError,'hard cap'):
                await client.call([{'role':'user','content':'test'}],request_id='R')
            self.assertFalse(evidence.records.get('model-calls'))
            evidence.close()
