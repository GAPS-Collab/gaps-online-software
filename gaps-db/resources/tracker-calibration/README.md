## Tracker calibration files
------------------------------

The following files seem to be needed for a calibrated tracker strip:

### "pedestals file" - pedestal adc value?

`-- layer row module channel 'val1' 'val2' ` 
`-- 0 0 0 0 121.899 2.889 `

* if a strip is missing, use the means for val1 and val2 for this strip
* I am making a wild guess here and assume val1 is the mean and val2 is the 
  sigma for the pedestal distribution

