"""Synthetic publication-link checks; no measured inputs or runtime execution."""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import time
import unittest
import sys
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
SOURCE=HERE.parents[2]/'check_active_markdown_links.py'
spec=importlib.util.spec_from_file_location('publication_links',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
sha=lambda b:hashlib.sha256(b).hexdigest()
enc=lambda v:json.dumps(v,sort_keys=True).encode()

class Checks(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='via-link-fake-')
        self.root=Path(self.temp.name).resolve();self.package=self.root/'package';self.package.mkdir()
        self.old=Path('/authored/original/repo');self.campaign=self.root/'results/test'
        self.campaign.mkdir(parents=True);self.review=self.campaign/'review.md'
        self.payloads={'results/test/freeze.json':b'{}','results/test/raw.json':b'AUTHORED FAKE',
            'results/test/review.md':b'[evidence](/authored/original/repo/results/test/raw.json)\n'}
        fr=self.payloads['results/test/freeze.json']
        self.mapping={'artifacts':{'formal_freeze':{'path':str(self.old/'results/test/freeze.json'),
            'sha256':sha(fr),'bytes':len(fr)}}}
        self.payloads['results/test/mapping.json']=enc(self.mapping)
        for rel in ('results/test/freeze.json','results/test/review.md','results/test/mapping.json'):
            (self.root/rel).write_bytes(self.payloads[rel])
        self.inventory={'included_files':[{'path':p,'kind':'FILE','size':len(b),'sha256':sha(b)}
             for p,b in self.payloads.items()], 'provenance':{'records':[]}}
        for role,rel in [('joint441_final_freeze','results/test/freeze.json'),('generation_mapping','results/test/mapping.json')]:
            b=self.payloads[rel];self.inventory['provenance']['records'].append(
                {'role':role,'path':rel,'sha256':sha(b),'bytes':len(b)})
        archive=io.BytesIO()
        with tarfile.open(fileobj=archive,mode='w:gz') as tf:
            for rel,b in self.payloads.items():
                info=tarfile.TarInfo(rel);info.size=len(b);tf.addfile(info,io.BytesIO(b))
        self.part=archive.getvalue();(self.package/'part0001').write_bytes(self.part)
        self.patches=[patch.object(m,'ROOT',self.root),patch.object(m,'CURRENT_EVIDENCE',self.root/'results'),
            patch.object(m,'COMMON15_PACKAGE',self.package),patch.object(m,'COMMON15_CAMPAIGNS',(self.campaign,)),
            patch.object(m,'FOUR_DP_BUNDLE',self.root/'absent-old-bundle'),
            patch.object(m,'ACTIVE_INPUTS',(self.review,))]
        for p in self.patches:p.start()
        self.resign()

    def tearDown(self):
        for p in reversed(self.patches):p.stop()
        self.temp.cleanup()

    def resign(self):
        inventory=enc(self.inventory);(self.package/'PRESERVATION-INVENTORY.json').write_bytes(inventory)
        manifest={'inventory_sha256':sha(inventory),'archive_bytes':len(self.part),
            'archive_sha256':sha(self.part),'parts':[{'name':'part0001','bytes':len(self.part),'sha256':sha(self.part)}]}
        b=enc(manifest);(self.package/'chunk-manifest.json').write_bytes(b)
        m.COMMON15_MANIFEST_SHA=sha(b)  # Synthetic trust anchor only, no real input modified.

    def result(self):
        with contextlib.redirect_stdout(io.StringIO()):return m.main()

    def test_valid_packaged_only_target_and_original_root(self):
        self.assertFalse((self.campaign/'raw.json').exists())
        self.assertEqual(self.result(),0)

    def test_corrupt_part_fails(self):
        (self.package/'part0001').write_bytes(b'x'*len(self.part));self.assertEqual(self.result(),1)

    def test_missing_part_fails(self):
        (self.package/'part0001').unlink();self.assertEqual(self.result(),1)

    def test_missing_inventory_fails(self):
        (self.package/'PRESERVATION-INVENTORY.json').unlink();self.assertEqual(self.result(),1)

    def test_unbound_manifest_fails(self):
        (self.package/'chunk-manifest.json').write_bytes(b'{}');self.assertEqual(self.result(),1)

    def test_unbound_mapping_role_fails(self):
        self.inventory['provenance']['records'][1]['path']='../outside.json';self.resign()
        self.assertEqual(self.result(),1)

    def test_duplicate_inventory_member_fails(self):
        self.inventory['included_files'].append(self.inventory['included_files'][0]);self.resign()
        self.assertEqual(self.result(),1)

    def test_unregistered_missing_active_link_still_fails(self):
        active=self.root/'README.md';active.write_text('[missing](results/test/not-in-package.json)\n')
        with patch.object(m,'ACTIVE_INPUTS',(active,)):self.assertEqual(self.result(),1)

    def test_changed_preserved_markdown_fails(self):
        self.review.write_text('Changed preserved review\n');self.assertEqual(self.result(),1)

    def test_outside_original_root_still_fails(self):
        self.review.write_text('[outside](/unrelated/private/file.json)\n')
        self.assertEqual(self.result(),1)

    def test_inside_root_broken_symlink_cannot_expand_membership(self):
        (self.campaign/'raw.json').symlink_to(self.root/'not-in-package.json')
        active=self.root/'README.md';active.write_text('[missing](not-in-package.json)\n')
        with patch.object(m,'ACTIVE_INPUTS',(active,)):self.assertEqual(self.result(),1)

if __name__=='__main__':
    label=sys.argv[1] if len(sys.argv)>1 else 'attempt-01'
    if label not in ('attempt-01','attempt-02'):raise ValueError('Explicit synthetic attempt required')
    start=time.monotonic();r=unittest.TextTestRunner(verbosity=1).run(unittest.defaultTestLoader.loadTestsFromTestCase(Checks))
    record={'scope':'SYNTHETIC_PUBLICATION_METADATA_ONLY','tests':r.testsRun,'failures':len(r.failures),
        'errors':len(r.errors),'seconds':time.monotonic()-start,'source_sha256':sha(SOURCE.read_bytes()),
        'measured_inputs_read':0,'model_calls':0,'analyzer_or_runtime_calls':0}
    with (HERE/('fake-results-'+label+'.json')).open('x') as f:json.dump(record,f,indent=2);f.write('\n')
    raise SystemExit(0 if r.wasSuccessful() else 1)
