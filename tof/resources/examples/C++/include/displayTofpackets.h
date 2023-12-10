#include "./Waveplot.h"
#include "./MainFrame.h"

/* Prototypes */
void displayData(char *root_file, char *parameter_file, int gui_flag);
void loadParameters(char *parameter_file, struct Limits *l);
int gui_wait();
//void plot(int n_ch, int ch_start, Vec<Waveform> &wave, Vec<Waveform> &wch9);
void plotrb(int n_ch, int ch_start);
void plotall(int n_ch, int nrbs);
void read_events(void);
int  event_flag(int evno);
extern void InitGui();

/* Constants */
#define MAX_EVENTS 1000
enum GUISWITCH { WAITING, NEXT, QUIT, ALLCH, SELECT, PRINT, FINISH,
                 RESTRICT, FIT };

