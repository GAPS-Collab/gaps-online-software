"""
Interaction with telemetry stream & packets.
"""

telemetry_import_success = False
try:
  import go_pybindings
  tl = go_pybindings.telemetry
  telemetry_import_success = True
except ImportError as e:
    print(e)
    print('--> Pybindings for telemetry-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')


def save_unpack_merged_event(pack):
    """
    Save unpack of a TelemetryPacket
    """
    ev = tl.MergedEvent()
    try:
        ev.from_telemetrypacket(pack)
    except Exception:
        return False
    try:
        ev.tof
    except Exception:
        return False
    return ev
