# Pre-request production and source-activity diagnostics

QA01/03 in these serialized fixtures measure requests after preceding source production is quiescent. Source changes, including correction, finish their producer work before the next STIMULUS. The producer is not free: usage/cost is already included in QA05; observed preparation time is reported here separately. No immediate-request-during-production behavior or counterfactual latency is measured. Source/activity-to-next-display spans include all intervening authored activities and producer work.

|Phase / case / option|Runs|Pre-request model calls|Mean pre-request model ms|Pre-request model $|Source/activity → next display mean ms|
|---|---:|---:|---:|---:|---:|
|BASIC dp01-e1 A|3|0|0.000|0.000000|6518.164|
|BASIC dp01-e2 A|3|0|0.000|0.000000|7404.927|
|BASIC dp01-e3 A|3|0|0.000|0.000000|8326.457|
|BASIC dp01-e4 A|3|0|0.000|0.000000|10090.683|
|BASIC dp01-e1 B|3|0|0.000|0.000000|6845.066|
|BASIC dp01-e2 B|3|0|0.000|0.000000|7306.699|
|BASIC dp01-e3 B|3|0|0.000|0.000000|8254.662|
|BASIC dp01-e4 B|3|0|0.000|0.000000|9462.375|
|BASIC dp01-e1 C|3|3|21608.542|0.006856|26628.805|
|BASIC dp01-e2 C|3|12|93298.969|0.030549|65129.404|
|BASIC dp01-e3 C|3|9|64935.500|0.020554|39791.584|
|BASIC dp01-e4 C|3|9|56980.310|0.018357|52846.975|
|BASIC dp05-e1 A|3|0|0.000|0.000000|0.045|
|BASIC dp05-e2 A|3|0|0.000|0.000000|0.057|
|BASIC dp05-e3 A|3|0|0.000|0.000000|0.041|
|BASIC dp05-e4 A|3|0|0.000|0.000000|0.091|
|BASIC dp05-e1 B|3|9|35495.103|0.013153|24660.470|
|BASIC dp05-e2 B|3|12|58991.751|0.022151|40895.452|
|BASIC dp05-e3 B|3|12|51714.819|0.020033|34411.982|
|BASIC dp05-e4 B|3|18|108858.007|0.061827|36773.970|
|BASIC dp05-e1 C|3|0|0.000|0.000000|0.025|
|BASIC dp05-e2 C|3|0|0.000|0.000000|0.044|
|BASIC dp05-e3 C|3|0|0.000|0.000000|0.054|
|BASIC dp05-e4 C|3|0|0.000|0.000000|0.077|
|SMOKE dp01-e1 C|1|1|21575.328|0.002289|26767.733|
|SMOKE dp05-e2 C|1|0|0.000|0.000000|0.072|
|TACTIC dp01-e3 A|3|0|0.000|0.000000|7874.405|
|TACTIC dp05-e3 C|3|0|0.000|0.000000|0.094|
