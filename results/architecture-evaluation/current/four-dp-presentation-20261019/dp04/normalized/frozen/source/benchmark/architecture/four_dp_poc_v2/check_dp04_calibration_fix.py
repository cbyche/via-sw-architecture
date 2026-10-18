"""Run existing offline structural checks on the isolated calibration-fix scratch."""
from pathlib import Path
import run
run.CRATE=Path('/private/tmp/four_dp_poc_v2_dp04_calibration_fix')
run.DEFAULT=run.DEFAULT/'analysis/dp04-calibration-fix-v2'
source=Path(__file__).with_name('check_dp04_contract.py')
exec(compile(source.read_text(),str(source),'exec'),{'__name__':'__main__','__file__':str(source)})
