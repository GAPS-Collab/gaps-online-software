"""
Training of the searchlight selection

Searchlight picks the primary among its candidate lines with a
logistic regression over the features of the candidates
(SearchlightResult.candidates). The weights built into gondola-core
(SearchlightSettings.DEFAULT_WEIGHTS, in Rust) are trained on MC.
With this module they can be retrained, e.g. on a larger sample:

    from gondola.reconstruction import Searchlight, SearchlightGeometry, SearchlightSettings
    from gondola.reconstruction import searchlight_training as st

    searchlight = Searchlight(geometry, SearchlightSettings())
    results, truths = [], []
    for event, true_direction in mc_events:       # TelemetryEvent, unit vector
        results.append(searchlight.reconstruct(event))
        truths.append(true_direction)
    X, y    = st.collect(results, truths)
    weights = st.train(X, y)
    st.save_weights('searchlight-weights.json', weights, settings=searchlight.settings,
                    note='antiproton MC, 50k events')

    settings = SearchlightSettings()
    settings.weights = st.load_weights('searchlight-weights.json')

The features do not depend on the weights, so the results for the
training can come from any weights. The weights belong to the
configuration they are trained with (independent_vertex,
use_tracks, use_timing, ...), which save_weights stores along.
"""

import json
import datetime

import numpy as np

# a candidate is good if its direction is within this angle (deg)
# of the true direction
GOOD_ANGLE = 8.0


def angle(a, b):
    """
    Angle between two directions in degree, without their sign
    """
    a = np.asarray(a, dtype=float)
    b = np.asarray(b, dtype=float)
    c = abs(np.dot(a, b))/(np.linalg.norm(a)*np.linalg.norm(b))
    return float(np.degrees(np.arccos(min(c, 1.0))))


def collect(results, true_directions, good_angle=GOOD_ANGLE):
    """
    The training sample: the features of the candidates which took
    part in the selection, and if they are good

    # Arguments:
        * results         : SearchlightResults
        * true_directions : the true direction of the primary for
                            each result
        * good_angle      : a candidate is good if it is within
                            this angle (deg) of the true direction

    # Returns:
        * X : features, one row per candidate
        * y : 1 for good candidates, 0 otherwise
    """
    X, y = [], []
    for res, u in zip(results, true_directions):
        for kind, direction, prob, features, allowed in res.candidates:
            if not allowed:
                continue
            X.append(features)
            y.append(1.0 if angle(direction, u) < good_angle else 0.0)
    return np.array(X, dtype=float), np.array(y, dtype=float)


def train(X, y, l2=1e-2, iterations=3000, step=0.1):
    """
    Logistic regression by gradient descent on standardized
    features (the first feature is the constant 1)

    # Arguments:
        * X          : features, one row per candidate
        * y          : 1 for good candidates, 0 otherwise
        * l2         : strength of the L2 penalty (not on the
                       constant)
        * iterations : number of gradient steps
        * step       : step size

    # Returns:
        * the weights for the raw features, as a list
    """
    X  = np.asarray(X, dtype=float)
    y  = np.asarray(y, dtype=float)
    mu = X.mean(0)
    sd = X.std(0)
    mu[0] = 0
    sd[0] = 1
    sd[sd == 0] = 1
    Z = (X - mu)/sd
    w = np.zeros(X.shape[1])
    for _ in range(iterations):
        p  = 1/(1 + np.exp(-Z @ w))
        g  = Z.T @ (p - y)/len(y) + l2*np.r_[0, w[1:]]
        w -= step*g
    # back to the raw features
    wr    = w/sd
    wr[0] = w[0] - np.sum(w[1:]*mu[1:]/sd[1:])
    return [float(x) for x in wr]


def save_weights(path, weights, settings=None, note=''):
    """
    Store weights as json, with the configuration they belong to

    # Arguments:
        * path     : file name
        * weights  : the weights (e.g. from train)
        * settings : SearchlightSettings used for the training
        * note     : free text, e.g. the sample
    """
    doc = {'weights'  : [float(w) for w in weights],
           'created'  : datetime.datetime.now().isoformat(timespec='seconds'),
           'note'     : note}
    if settings is not None:
        doc['configuration'] = {k : getattr(settings, k) for k in
                                ('edep_min', 'trk_e_min', 'vertex_err', 'use_vertex', 'use_tracker',
                                 'use_trigger', 'independent_vertex', 'use_tracks', 'use_timing')}
    with open(path, 'w') as f:
        json.dump(doc, f, indent=2)


def load_weights(path):
    """
    The weights from a file written by save_weights, to set them
    with settings.weights = load_weights(path)
    """
    with open(path) as f:
        return json.load(f)['weights']
