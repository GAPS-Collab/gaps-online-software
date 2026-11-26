import json
from pathlib import Path
import numpy as np

HERE = Path(__file__).resolve().parent
STATE_OF_THE_ART = HERE / "v2" / "model.txt"
# input name -> (TofHit field, transform)
ln, nan0, ident = np.log, (lambda v: np.nan_to_num(v, nan=0.0)), (lambda v: v)
TRANSFORMS = {"ln ob_pk_A": ("ob_pk_A", ln), "ln ob_q_A": ("ob_q_A", ln), "ob_tot550_A": ("ob_tot550_A", ident),
              "ob_tot725_A": ("ob_tot725_A", ident), "ob_rs550_A (NaN->0)": ("ob_rs550_A", nan0),
              "ob_rs725_A (NaN->0)": ("ob_rs725_A", nan0), "ob_ped_A": ("ob_ped_A", ident),
              "ob_pedrms_A": ("ob_pedrms_A", ident), "ln ob_pk_B": ("ob_pk_B", ln), "ln ob_q_B": ("ob_q_B", ln),
              "ob_tot550_B": ("ob_tot550_B", ident), "x_ob": ("x_ob", ident)}
FEATURES = list(TRANSFORMS)                                 # the 12 raw columns, v1 order
FIELDS = tuple(TRANSFORMS[f][0] for f in FEATURES)
V2_FEATURES = ["ob_tot725_A", "ob_rs550_A (NaN->0)", "ob_tot550_A", "ob_rs725_A (NaN->0)", "x_ob", "ln ob_q_A",
               "ln ob_q_B", "ob_pedrms_A"]

# training maximum of the end-A TOTs [ns] per version (frozen training set, train_v2.prep); above it the linear leaves
# extrapolate with nothing to check them against (v2_input_domain.py: 3.4 % of flight saturated ends)
TOT_MAX = {"v2": {"ob_tot725_A": 56.28125, "ob_tot550_A": 61.125}}


def features(h, names=FEATURES):
    return np.column_stack([TRANSFORMS[f][1](np.asarray(h[TRANSFORMS[f][0]], float)) for f in names])


class SatCorrector:
    def __init__(self, path=STATE_OF_THE_ART):
        import lightgbm as lgb
        self.path = Path(path)
        self.booster = lgb.Booster(model_file=str(self.path))
        fj = self.path.parent / "features.json"
        self.names = json.loads(fj.read_text()) if fj.exists() else FEATURES
        assert self.booster.num_feature() == len(self.names), (self.path, self.names)

    def predict(self, h):
        return np.exp(self.booster.predict(features(h, self.names)))

    def tot_in_range(self, h):
        """True where both end-A TOTs are within the training maximum of this version."""
        mx = TOT_MAX[self.path.parent.name]
        return np.all([np.asarray(h[TRANSFORMS[f][0]], float) <= m for f, m in mx.items()], axis=0)