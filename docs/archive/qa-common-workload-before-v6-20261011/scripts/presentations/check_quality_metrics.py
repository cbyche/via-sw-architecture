#!/usr/bin/env python3
"""Check the current six ASR-QA metrics; previous V checks are archived."""
from check_asr_qa_presentations import common_check,check
if __name__=='__main__':
    common_check()
    check('metrics')
