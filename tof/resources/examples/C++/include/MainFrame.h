#include <TGClient.h>
#include <TRootEmbeddedCanvas.h>
#include <TGButton.h>
#include <TGTextEntry.h>
#include <TGCanvas.h>
#include <TCanvas.h>
#include <TH1.h>
#include <TMarker.h>

class MainFrame : public TGMainFrame {
 private:
  TGHorizontalFrame *actionsFrame;
  TGTextButton  *fButton1;
  TGTextButton  *fButton2;
  TGTextButton  *fButton3;
  TGTextButton  *fButton4;
  TGTextButton  *fButton5;
  TGTextButton  *fButton6;
  TGTextEntry   *fEntry;
  TGCheckButton *fRange;
  TGLayoutHints *actionsLayout;
  TGLayoutHints *buttonsLayout;
  TGLayoutHints *fLayout;
  TGCompositeFrame     *fContainer;
  TGCanvas             *fCanvasWin;
  TRootEmbeddedCanvas  *fECanvas;
  char          fBuffer[10];
 public:
  //TCanvas *MainFrame::GetCanvas();
  //const char *MainFrame::GetText();
  TCanvas *GetCanvas();
  const char *GetText();
  MainFrame(const TGWindow *p, UInt_t w, UInt_t h);
  ~MainFrame();
  Bool_t ProcessMessage(Long_t msg, Long_t parm1, Long_t parm2);
  Int_t status;
};


