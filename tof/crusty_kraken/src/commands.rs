/*****************************
 *
 * Commands to be sent over
 * network to interact with 
 * the instance.
 *
 */


enum TOFCOMMAND {
    STOP_ACQUISTION     = 10,
    START_ACQUISITION   = 20,
    CALIBRATE           = 30,
    RESEND_LAST_PACKAGE = 40
}

