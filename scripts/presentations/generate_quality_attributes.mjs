// Compatibility entry point. Copy with generate_asr_qa_presentations.mjs into a fresh build directory.
process.env.VIA_QA_DECK_KIND='attributes';
await import('./generate_asr_qa_presentations.mjs');
