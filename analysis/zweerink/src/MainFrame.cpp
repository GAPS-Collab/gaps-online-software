#include "../include/MainFrame.h"

MainFrame::MainFrame(const TGWindow *p, UInt_t w, UInt_t h) : TGMainFrame(p,w,h)
{
  actionsFrame = new TGHorizontalFrame(this, 100, 20);
  fButton1 = new TGTextButton(actionsFrame, "Next Event", 1);
  fButton1->Resize(100,20);
  fButton1->Associate(this);
  fButton2 = new TGTextButton(actionsFrame, "Show All",  3);
  fButton2->Resize(100,20);
  fButton2->Associate(this);
  fButton3 = new TGTextButton(actionsFrame, "Print", 5);
  fButton3->Resize(100,20);
  fButton3->Associate(this);
  fButton4 = new TGTextButton(actionsFrame, "Finish", 6);
  fButton4->Resize(100,20);
  fButton4->Associate(this);
  fButton5 = new TGTextButton(actionsFrame, "Exit", 2);
  fButton5->Resize(100,20);
  fButton5->Associate(this);
  fButton6 = new TGTextButton(actionsFrame, "Fit", 8);
  fButton6->Resize(100,20);
  fButton6->Associate(this);
  fEntry   = new TGTextEntry(this,fBuffer,4); 
  fEntry->Resize(50, fEntry->GetDefaultHeight());
  fEntry->SetFont("-adobe-courier-bold-r-*-*-14-*-*-*-*-*-iso8859-1");
  fRange = new TGCheckButton(actionsFrame, "Restrict Range", 7);
  fRange->Resize(100,20);
  fRange->Associate(this);
  actionsLayout = new TGLayoutHints(kLHintsTop | kLHintsLeft);
  actionsFrame->AddFrame(fEntry, actionsLayout);
  actionsFrame->AddFrame(fButton1, actionsLayout);
  actionsFrame->AddFrame(fButton2, actionsLayout);
  actionsFrame->AddFrame(fButton3, actionsLayout);
  actionsFrame->AddFrame(fButton4, actionsLayout);
  actionsFrame->AddFrame(fButton5, actionsLayout);
  actionsFrame->AddFrame(fButton6, actionsLayout);
  actionsFrame->AddFrame(fRange, actionsLayout);
  buttonsLayout = new TGLayoutHints(kLHintsTop | kLHintsLeft);
  AddFrame(actionsFrame, buttonsLayout);
  //AddFrame(fEntry, buttonsLayout);
  
  fECanvas   = new TRootEmbeddedCanvas("JAH",this,w-40,h-40,0);
  fContainer = new TGCompositeFrame(fECanvas->GetViewPort(), 10, 10,
				    kHorizontalFrame, GetWhitePixel());
  fContainer->SetLayoutManager(new TGTileLayout(fContainer, 8));
  fECanvas->SetContainer(fContainer);

  fLayout= new TGLayoutHints(kLHintsBottom | kLHintsExpandX |
			     kLHintsExpandY, 2, 2, 5, 1);
  AddFrame(fECanvas,fLayout);

  MapSubwindows();
  Layout();
  SetWindowName("Waveform Display");
  SetIconName("Waveform Display");
  MapWindow();

}

MainFrame::~MainFrame() 
{

  delete actionsFrame;
  delete fButton1;
  delete fButton2;
  delete fButton3;
  delete fButton4;
  delete fButton5;
  delete fButton6;
  delete fRange;
  delete fEntry;
  delete actionsLayout;
  delete buttonsLayout;
  delete fLayout;
  delete fContainer;
  delete fCanvasWin;
  delete fECanvas;
    
}

Bool_t MainFrame::ProcessMessage(Long_t msg, Long_t parm1, Long_t parm2)
{

  //  printf("PM %ld %ld %ld\n",msg,parm1,parm2);

  switch (GET_MSG(msg)) {
  case kC_COMMAND:
    status = parm1;
    break;
  case kC_TEXTENTRY:
    switch (GET_SUBMSG(msg)) {
    case kC_TEXTENTRY:
      break;
    case kTE_ENTER:
      status = parm1;
      break; 
    default:
      break;
    }
    break;
  default:
    status=0;
    break;
  }
  
  return(kTRUE);
}

TCanvas *MainFrame::GetCanvas() {

  return(fECanvas->GetCanvas());
  
}

const char *MainFrame::GetText() {
  
  return(fEntry->GetText());
  
}


