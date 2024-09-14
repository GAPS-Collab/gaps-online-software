import gaps_online.telemetry as tl

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