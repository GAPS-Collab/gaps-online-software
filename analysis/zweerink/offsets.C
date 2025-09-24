// Macro that reads a bunch of histograms with the residual hit times
// between various pairs of paddles and calculates offsets for those
// paddles.

#include "TInterpreter.h"
#include "TCanvas.h"
#include "TSystem.h"
#include "TFile.h"
#include "TH2.h"
#include "TNtuple.h"
#include "TPaveLabel.h"
#include "TPaveText.h"
#include "TFrame.h"
#include "TSystem.h"
#include "TInterpreter.h"

// Some useful macros
#define SQR(A)               ( (A) * (A) )
#define ABS(A)               ( ( (A<0) ? -(A) : (A) ) )
#define PI                   3.14159265
// In units of mm/ns
#define CSPEED               299.792458

double OffsetFunc(double a[4][4], double b[4][4], double par[8]);


// These should match the values in
// gaps-online-software/tof/resources/examples/C++/include/EventGAPS.h
// because that is where I define the histogram arrays. There is a mix
// of notation here. Each of the paddles is numbered like GAPS (1-160)
// but all the arrays used in this program all start at zero (except
// for offsets[]). CUB-Top 1-12, CUB-Bot 13-24, CUB-Sides 25-60,
// UMB-Center 61-72, UMB 61-108, CORT 109-160.

const int NPAD    = 161;
const int NFILES  =  2;
const int NUMBC   = 12;
const int NCUBT   = 60;

const int NCUBC   = 12;
const int NUMBT   = 48;

const int NCUBS   =  8;
const int NCORT   = 10;

#define PRINT_FITS  0
#define PRINT_DIFFS 0

void offsets() {

  char text[500];

  float offset[NPAD] = { 0.0 };
  
  float CUBampl[NUMBC][NCUBT]; // Amplitude from analyzeNevis hist.
  float CUBmean[NUMBC][NCUBT]; // Mean from analyzeNevis hist.
  float CUBsdev[NUMBC][NCUBT]; // StdDev from analyzeNevis hist.
  int   CUBnent[NUMBC][NCUBT]; // Num Entries from analyzeNevis hist.
  
  float UMBampl[NCUBC][NUMBT]; // Amplitude from analyzeNevis hist.
  float UMBmean[NCUBC][NUMBT]; // Mean from analyzeNevis hist.
  float UMBsdev[NCUBC][NUMBT]; // StdDev from analyzeNevis hist.
  int   UMBnent[NCUBC][NUMBT]; // Num Entries from analyzeNevis hist.

  float CORampl[NCUBS][4*NCORT]; // Amplitude from analyzeNevis hist.
  float CORmean[NCUBS][4*NCORT]; // Mean from analyzeNevis hist.
  float CORsdev[NCUBS][4*NCORT]; // StdDev from analyzeNevis hist.
  int   CORnent[NCUBS][4*NCORT]; // Num Entries from analyzeNevis hist.

  // Filename produced when running analyzeNevis
  
  char fname[500] = "test134.root";
  //char fname[500] = "test144.root";
  //char fname[500] = "offset_134Find.root";
  //char fname[500] = "offset_144Find.root";
  //char fname[500] = "offset_144Test.root";
  
  TFile *fp;

  TH1F *h1, *h;

  TCanvas *BvT, *b;
  b = (TCanvas*)gROOT->FindObject("BvT");if (b) {b->Close(); delete b; b=0;}
  //BvT = new TCanvas("BvT", "BvT", 100, 0, 1000, 1000);
  //BvT->Divide(3,4);

  TF1 *f, *fS;

  // Now, find the relevant histos and calculate the offsets
  fp = new TFile(fname);

  // Little inelegant, but we will deal with three sets of
  // offsets. First, the 12 Umb-Center and all the Cube paddles
  if (PRINT_FITS) printf("Now reading CUBE offset histos\n");
  for (int i=0;i<NCUBT;i++) {
    for(int j=0;j<NUMBC;j++) {
      snprintf(text, 256, "H_OffCube[%d][%d]", i+1, j);

      h = (TH1F*)gROOT->FindObject(text); if (h) h->Delete(); h=0;
      h1 = (TH1F*)fp->Get(text);
      if (h1->GetEntries() > 10) {
	h1->SetDirectory(0);
	f = (TF1*)gROOT->FindObject("fS"); if (f) f->Delete(); delete f; f=0;
	float hmean = h1->GetMean();;
	float hmax  = h1->GetBinCenter(h1->GetMaximumBin());
	TF1 *fS = new TF1("fS", "gaus", hmax-0.6, hmax+0.6);
	fS->SetParameters(h1->GetEntries()/10.0, hmean, h1->GetStdDev()); 
	//BvT->cd(j+1);
	h1->Fit(fS, "qR0");

	float amp  = fS->GetParameter(0);
	float mean = fS->GetParameter(1);
	float width = fS->GetParameter(2);
	int cube = i + 1;
	int umb  = j + 61;
	int Numb = j;
	int Ncube = i;
	CUBmean[Numb][Ncube] = mean;
	CUBampl[Numb][Ncube] = amp;
	CUBsdev[Numb][Ncube] = width;
	CUBnent[Numb][Ncube] = h1->GetEntries();
	//if (i%12>=3 && i%12<9) {
	if (j==0) h1->SetAxisRange(0.0, 5.0, "X");
	if (j==1) h1->SetAxisRange(5.0, 9.0, "X");
	//h1->GetXaxis()->SetTitle("Time-of-Flight");
	//h1->Draw("same");
	if (PRINT_FITS) 
	  printf("%3d %3d %7.0f %6.3f %6.3f -- %7.2f %6.3f %6.3f\n", umb, cube,
		h1->GetEntries(),h1->GetMean(),h1->GetStdDev(), amp,mean,width);
	fflush(stdout);
	fS->Delete();
      }
      char input[100];
      if (j==11 && 0) {
	BvT->Update();
	fscanf(stdin,"%s",input);
	if (strncmp(input,"q",1) == 0) return;
	else if (strncmp(input,"p",1) ==0 ) BvT->Print("histos.pdf");
      }
    }
  }

  if (PRINT_FITS) printf("Now reading UMB offset histos\n");
  // Now for the 12 Cube-Center and all the Umbrella paddles
  for (int i=0;i<NUMBT;i++) {
    for(int j=0;j<NCUBC;j++) {
      snprintf(text, 256, "H_OffUmb[%d][%d]", i, j);

      int cube = j;
      int umb  = i + 61;
      int Numb = i;
      int Ncube = j;
      
      h = (TH1F*)gROOT->FindObject(text); if (h) h->Delete(); h=0;
      h1 = (TH1F*)fp->Get(text);
      //if (h1) printf("%3d %3d: %.2lf  ", umb, cube, h1->GetEntries() );
      if (h1->GetEntries() > 10) {
	h1->SetDirectory(0);
	f = (TF1*)gROOT->FindObject("fS"); if (f) f->Delete(); delete f; f=0;
	float hmean = h1->GetMean();;
	float hmax  = h1->GetBinCenter(h1->GetMaximumBin());
	TF1 *fS = new TF1("fS", "gaus", hmax-0.6, hmax+0.6);
	fS->SetParameters(h1->GetEntries()/10.0, hmean, h1->GetStdDev()); 
	//BvT->cd(j+1);
	h1->Fit(fS, "qR0");

	float amp  = fS->GetParameter(0);
	float mean = fS->GetParameter(1);
	float width = fS->GetParameter(2);
	UMBmean[Ncube][Numb] = mean;
	UMBampl[Ncube][Numb] = amp;
	UMBsdev[Ncube][Numb] = width;
	UMBnent[Ncube][Numb] = h1->GetEntries();
	//if (i%12>=3 && i%12<9) {
	if (j==0) h1->SetAxisRange(0.0, 5.0, "X");
	if (j==1) h1->SetAxisRange(5.0, 9.0, "X");
	//h1->Draw("same");
	if (PRINT_FITS) 
	  printf("%3d %3d %7.0f %6.3f %6.3f -- %7.2f %6.3f %6.3f\n", cube, umb,
	       h1->GetEntries(),h1->GetMean(),h1->GetStdDev(), amp,mean,width);
	fflush(stdout);
	fS->Delete();
      }
      char input[100];
      if (j==11 && 0) {
	BvT->Update();
	fscanf(stdin,"%s",input);
	if (strncmp(input,"q",1) == 0) return;
	else if (strncmp(input,"p",1) ==0 ) BvT->Print("histos.pdf");
      }
    }
  }

  if (PRINT_FITS) printf("Now reading CORT offset histos\n");
  // Now for the Cortina/Cube Side paddles
  for (int k=0; k<4; k++) { // 4 cortina sides, so 4 sets of histos
  for (int i=0;i<NCORT;i++) {
    for(int j=0;j<NCUBS;j++) {

      int cube = j + k*NCUBS + 25;
      int cor  = i + 109 + k*NCORT;
      int Ncor = i + k*NCORT;
      int Ncube = j;

      if (k==0) snprintf(text, 256, "H_OffCorN[%d][%d]", i, j);
      if (k==1) snprintf(text, 256, "H_OffCorE[%d][%d]", i, j);
      if (k==2) snprintf(text, 256, "H_OffCorS[%d][%d]", i, j);
      if (k==3) snprintf(text, 256, "H_OffCorW[%d][%d]", i, j);
      
      h = (TH1F*)gROOT->FindObject(text); if (h) h->Delete(); h=0;
      h1 = (TH1F*)fp->Get(text);
      //if (h1) printf("%3d %3d: %.2lf  ", cor, cube, h1->GetEntries() );
      if (h1->GetEntries() > 10) {
	h1->SetDirectory(0);
	f = (TF1*)gROOT->FindObject("fS"); if (f) f->Delete(); delete f; f=0;
	float hmean = h1->GetMean();;
	float hmax  = h1->GetBinCenter(h1->GetMaximumBin());
	TF1 *fS = new TF1("fS", "gaus", hmax-0.6, hmax+0.6);
	fS->SetParameters(h1->GetEntries()/10.0, hmean, h1->GetStdDev()); 
	//BvT->cd(j+1);
	h1->Fit(fS, "qR0");

	float amp  = fS->GetParameter(0);
	float mean = fS->GetParameter(1);
	float width = fS->GetParameter(2);
	CORmean[Ncube][Ncor] = mean;
	CORampl[Ncube][Ncor] = amp;
	CORsdev[Ncube][Ncor] = width;
	CORnent[Ncube][Ncor] = h1->GetEntries();
	//if (i%12>=3 && i%12<9) {
	if (j==0) h1->SetAxisRange(0.0, 5.0, "X");
	if (j==1) h1->SetAxisRange(5.0, 9.0, "X");
	//h1->Draw("same");
	if (PRINT_FITS) 
	  printf("%3d %3d %7.0f %6.3f %6.3f -- %7.2f %6.3f %6.3f\n", cube, cor,
	       h1->GetEntries(),h1->GetMean(),h1->GetStdDev(), amp,mean,width);
	fflush(stdout);
	fS->Delete();
      }
      char input[100];
      if (j==11 && 0) {
	BvT->Update();
	fscanf(stdin,"%s",input);
	if (strncmp(input,"q",1) == 0) return;
	else if (strncmp(input,"p",1) ==0 ) BvT->Print("histos.pdf");
      }
    }
  }
  }

  fp->Close();
  
  /////////////////////////////////////////////////////////////////
  // Now, calculate the CUB offsets using the UMB-Center 12 paddles. 
  /////////////////////////////////////////////////////////////////
  
  float off_umb[NUMBC] = { 0.0 };
  float off_cub[NCUBT] = { 0.0 };
  float sum_off[NUMBC] = { 0.0 };
  int   sum_ctr[NUMBC] = { 0 };
  
  // There are 72 offsets we need to calculate because the UMB-center
  // has 12 paddles and the CUBE has 60. Set all the CUBE offsets from
  // tracks hitting the U_REF paddle. Then, find the offsets for the
  // UMB paddles by looking at the residuals 

  // Set initial offsets for the UMB paddles with reference to UMB paddle 6 
  float diff;
  const int U_REF    = 5;
  for (int i=0;i<NCUBT;i++){
    if (PRINT_DIFFS) printf("%3d:    ", i+1);
    // Calculate the UMB offsets by finding the variance from the
    // expected value compared to channel 5
    for (int j=0;j<NUMBC;j++) {
      if (CUBsdev[j][i]>0.05 && CUBsdev[j][i]<0.4 &&
	  CUBsdev[U_REF][i]>0.05 && CUBsdev[U_REF][i]<0.4) {// good stddevs
        diff = (CUBmean[j][i]-CUBmean[U_REF][i]);
	if (PRINT_DIFFS) printf(" %6.3f", diff );
	if ( i>0 && i<25) { // Only use TOP/BOT paddles
	  sum_off[j] += diff;
	  sum_ctr[j]++;
	}
      } else if (PRINT_DIFFS) printf("       ");
    }      
    if (PRINT_DIFFS) printf("\n");
  }

  printf("\n61-72:   ");
  for (int j=0;j<NUMBC;j++) {
    if (sum_ctr[j]>0) {
      off_umb[j] = sum_off[j]/(float)sum_ctr[j];
      offset[j+61] = off_umb[j]; // 61-72
    } 
    printf("%6.3f,", off_umb[j]);
  }

  // Set offsets for the cube paddles. 
  for (int i=0;i<NCUBT;i++){
    if (i==0) printf("\n 1-12:   ");
    if (i==12) printf("\n13-24:   ");
    if (i==24 || i==32 || i==40 || i==48) printf("\n%d-%d:   ",i,i+7);
    if (i==56) printf("\n57-60:   ");
    int j=U_REF;  
    if (CUBsdev[j][i]>0.05 && CUBsdev[j][i]<0.4) { // Means with good stddevs
      // Find CUB paddle offset and add it to the UMB offset. 
      off_cub[i] = CUBmean[U_REF][i];
    } else {
      off_cub[i] = 0.0;
    }
    offset[i+1] = off_cub[i]; // 1-60
    printf("%6.3f,", off_cub[i]);
  }
  printf("\n");

  /////////////////////////////////////////////////////////////////
  // Now, calculate the UMB offsets using the CUB-Top 12 paddles. 
  /////////////////////////////////////////////////////////////////
  
  float off_umb1[NUMBT] = { 0.0 };
  float off_cub1[NCUBC] = { 0.0 };
  float sum_off1[NCUBC] = { 0.0 };
  int   sum_ctr1[NCUBC] = { 0 };
  
  // There are 60 offsets we need to calculate because the CUB-Top
  // has 12 paddles and the UMB has 48. Set all the UMB offsets from
  // tracks hitting the C_REF paddle. Then, find the offsets for the
  // CUB-Top paddles by looking at the residuals 

  // Set initial offsets for the UMB paddles with reference to CUB paddle 6 
  float diff1;
  const int C_REF    = 5;
  for (int i=0;i<NUMBT;i++){
    if (PRINT_DIFFS) printf("%3d:    ", i+61);
    // Calculate the CUB-Top offsets by finding the variance from the
    // expected value compared to channel C_REF
    for (int j=0;j<NCUBC;j++) {
      if (UMBsdev[j][i]>0.05 && UMBsdev[j][i]<0.4) { // means w/good stddevs
        diff1 = (UMBmean[j][i]-UMBmean[C_REF][i]);
	if (PRINT_DIFFS) printf(" %6.3f", diff1 );
	if ( i>=0 && i<NUMBC) { // Only use UMB-Center paddles
	  sum_off1[j] += diff1;
	  sum_ctr1[j]++;
	}
      } else if (PRINT_DIFFS) printf("       ");
    }      
    if (PRINT_DIFFS) printf("\n");
  }
  printf("\n");

  printf("1-12:    ");
  for (int j=0;j<NCUBC;j++) {
    if (sum_ctr1[j]>0) {
      off_cub1[j] = sum_off1[j]/(float)sum_ctr1[j];
    } 
    printf("%6.3f,", off_cub1[j]);
  }
  printf("\n");

  // Set offsets for the UMB paddles. 
  for (int i=0;i<NUMBT;i++){
    if (i==0) printf("61-72:   ");
    if (i==12) printf("\n73-84:   ");
    if (i==24) printf("\n85-96:   ");
    if (i==36) printf("\n97-108:  ");
    int j=C_REF;  
    if (UMBsdev[j][i]>0.05 && UMBsdev[j][i]<0.4) { // Means with good stddevs
      // Find UMB paddle offset and add it to the CUB-Top offset. 
      off_umb1[i] = UMBmean[j][i];
    } else {
      off_umb1[i] = 0.0;
    }
    printf("%6.3f,", off_umb1[i]);
    if (i>11) offset[i+61] = off_umb1[i] - offset[61+U_REF]; // 73-108
    //if (i%NUMBT==NUMBT-1) printf("\n");
  } 
  printf("\n");

  /////////////////////////////////////////////////////////////////
  // Now, calculate the COR offsets using the proper CUB-Side paddles. 
  /////////////////////////////////////////////////////////////////
  
  // There are 18 offsets we need to calculate for each Cortina/Cube
  // side combination.  Set all the CORT offsets from tracks hitting
  // the CS_REF paddle. Then, find the offsets for the CUB-Side
  // paddles by looking at the residuals

  // Set initial offsets for the CORT paddles with reference to CUBS paddle 4 
  float diff2;
  const int CS_REF    = 2;

  for (int k=0;k<4;k++) { // For each side of the instrument
  // Initialize for each side
  float off_cub2[NCUBS] = { 0.0 };
  float off_cor2[NCORT] = { 0.0 };
  float sum_off2[NCUBS] = { 0.0 };
  int   sum_ctr2[NCUBS] = { 0 };
  
  for (int i=0;i<NCORT;i++){
    int i_cor = i+k*NCORT;
    if (PRINT_DIFFS) printf("%3d:    ", i_cor+109);
    // Calculate the CUB-Side offsets by finding the variance from the
    // expected value compared to channel CS_REF
    for (int j=0;j<NCUBS;j++) {
      if (CORsdev[j][i_cor]>0.05 && CORsdev[j][i_cor]<0.4 &&
	  CORsdev[CS_REF][i_cor]>0.05 && CORsdev[CS_REF][i_cor]<0.4) {
	// means w/good stddevs
        diff2 = (CORmean[j][i_cor]-CORmean[CS_REF][i_cor]);
	if (PRINT_DIFFS) printf(" %6.3f", diff2);
	if ( i>=0 && i<NCORT) { // Only use CORT paddles
	  sum_off2[j] += diff2;
	  sum_ctr2[j]++;
	}
      } else if (PRINT_DIFFS) printf("       ");
    }      
    if (PRINT_DIFFS) printf("\n");
  }

  /* This prints the residual differences for the CUB-Side paddles for
     each appropriate CORT paddle. However, it appears buggy since the
     differences vary quite a bit for each CUB-Side paddle and there
     are many missing paddles. I am not sure the trigger conditions
     give us good events that allow us to calculate good offsets for
     the CORT yet. However, all we need for the CORT offsets is the
     REFERENCE paddle residual.

  printf("\n%d-%d:   ", 25+k*NCUBS, 25+(k+1)*NCUBS-1);
  for (int j=0;j<NCUBS;j++) {
    if (sum_ctr2[j]>0) {
      off_cub2[j] = sum_off2[j]/(float)sum_ctr2[j];
    } 
    printf("%6.3f,", off_cub2[j]);
  }
  printf("\n");
  */
  
  // Set offsets for the COR paddles. 
  for (int i=0;i<NCORT;i++){
    int i_cor = i+k*NCORT;
    if (i==0) printf("%3d-%3d: ", 109+i_cor, 109+i_cor+9);
    int j=CS_REF;  
    if (CORsdev[j][i_cor]>0.05&&CORsdev[j][i_cor]<0.4) { // Means w/good stddevs
      // Find COR paddle offset and add it to the CUB-Sid offset. 
      off_cor2[i] = CORmean[j][i_cor];
    } else {
      off_cor2[i] = 0.0;
    }
    printf("%6.3f,", off_cor2[i]);
    offset[i_cor+109] = off_cor2[i] - offset[25+k*NCUBS+CS_REF]; // 109-160
    if (i%NCORT==NCORT-1) printf("\n");
  }
  }
  printf("\n");
  
  // All the stuff above this is used to calculate the offsets. Here,
  // we print the offsets in a C++-friendly way to copy into an array.
  printf("C++ format: copy/paste into a relevant 160-value array\n\n");
  
  for (int i=1; i<NPAD; i++) { 
    printf(" %6.3f,", offset[i]);
    if (i%8 == 0) printf("\n");
  }
  printf("\n");
  
}

int main(){
  char const *basename = "run144_Toff.root";
  offsets();
}

double OffsetFunc(double a[4][4], double b[4][4], double par[8]) {
  // Offset function                                                          

  double f=0;

  //We have 
  
  for ( int i=0;i<4;i++) {
    for ( int j=0;j<4;j++) {
      f+= ( a[i][j] - par[i] ) - par[4+j]; 
    }
  }
  return f;
}

