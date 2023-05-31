"""
DAQ channel identifier to position
"""
import numpy as np

def encode_14bit(layer, row, module, channel):
    """
    The strip ID using 14 bits: 3 bits for the layer, 3 bits for the row, 3 bits for the module, 5 bits for the channel
    Args:
            layer:
            row:
            module:
            channel:

    Returns:

    """
    return channel | module << 5 | row << 8 | layer << 11


def decode_14bit(number):
    layer = int(number & int("0b11100000000000", base=2)) >> 11
    row = int(number & int("0b00011100000000", base=2)) >> 8
    module = int(number & int("0b00000011100000", base=2)) >> 5
    channel = int(number & int("0b00000000011111", base=2))
    return layer, row, module, channel

# distance between layers
dZ_layer  = 100   # mm
# distance module-center detector-center
dX_det    = 57.65 # mm
# distance module-center module-center in x and y
dX_module = 120.65   # mm
# strip dx - distance of strip center from individual module
dX_strip_asc     = [-36.45, -23.16, -13.4,-4.4 , 4.4, 13.4, 23.15, 36.45 ]
dX_strip_desc    = [k for k in reversed(dX_strip_asc)]

def module_coordinates(row, mod, layer):
        """
        Return the absolute coordinate for the center of
        a module
        """
        layer_0_z = 1184  # mm
        if layer % 2 == 0:
                # module 0-0 sits at max (y) min (x)
                # ascending module in -y direction
                # ascending row in +x direction
                mod_0 = [-603, +603]  # mm
                mod_x = mod_0[0] + 2 * dX_module * row
                mod_y = mod_0[1] - 2 * dX_module * mod

        else:
                # module 0-0 sits at min (y) min (x)
                # ascending module in -x direction
                # ascending row in +y direction
                mod_0 = [603, -603]
                mod_x = mod_0[0] - 2 * dX_module * mod
                mod_y = mod_0[1] + 2 * dX_module * row
        mod_z = layer_0_z - layer * dZ_layer
        return np.array([mod_x, mod_y, mod_z])


def channel_coordinates(row, mod, layer, ch, only_detectors=False):
        """
        Return the absolute coordinate
        for the center of a strip

        Keyword Args:
            only_detectors (bool) : Return only the coordinates for the detectors,
                                    not the strips
        """
        mod_coord = module_coordinates(row, mod, layer)

        if layer % 2 == 0:
                # for EVEN layers, 'left' detectors (when looking from above)
                # are on the smaller y-side
                # the strips are 7-0  and 31-24 in

                left_side_hv = [k for k in range(8)]
                left_side_nohv = [k for k in range(24, 32)]
                right_side_hv = [k for k in range(8, 16)]
                right_side_nohv = [k for k in range(16, 24)]
                left_side = left_side_hv + left_side_nohv
                if ch in left_side:
                        det_y = mod_coord[1] - dX_det
                        strip_y = det_y
                        if ch in left_side_hv:
                                det_x = mod_coord[0] - dX_det
                                strip_x = det_x + dX_strip_asc[ch]
                        else:
                                det_x = mod_coord[0] + dX_det
                                strip_x = det_x + dX_strip_asc[ch - 24]

                else:
                        det_y = mod_coord[1] + dX_det
                        strip_y = det_y
                        if ch in right_side_hv:
                                det_x = mod_coord[0] - dX_det
                                strip_x = det_x + dX_strip_desc[ch - 8]
                        else:
                                det_x = mod_coord[0] + dX_det
                                strip_x = det_x + dX_strip_desc[ch - 16]

        else:
                # for ODD layers, 'left' detectors (when looking from above)
                # are CLOSER TO THE HV CONNECTOR and on the +x side
                # the strips are 7-0  and 8-15
                hv_side_xmin = [k for k in range(8)]
                hv_side_xmax = [k for k in range(8, 16)]
                hv_side = hv_side_xmax + hv_side_xmin
                no_hv_side_xmin = [k for k in range(24, 32)]
                no_hv_side_xmax = [k for k in range(16, 24)]
                if ch in hv_side:
                        det_y = mod_coord[1] - dX_det
                        strip_y = det_y
                        if ch in hv_side_xmax:
                                det_x = mod_coord[0] + dX_det
                                strip_y += dX_strip_desc[ch - 8]
                        else:
                                det_x = mod_coord[0] - dX_det
                                strip_y += dX_strip_desc[ch]
                        strip_x = det_x
                else:
                        det_y = mod_coord[1] + dX_det
                        strip_y = det_y
                        if ch in no_hv_side_xmax:
                                det_x = mod_coord[0] + dX_det
                                strip_y += dX_strip_asc[ch - 16]
                        else:
                                det_x = mod_coord[0] - dX_det
                                strip_y += dX_strip_asc[ch - 24]
                        strip_x = det_x
        if only_detectors:
                return np.array([det_x, det_y, mod_coord[2]])
        return np.array([strip_x, strip_y, mod_coord[2]])
