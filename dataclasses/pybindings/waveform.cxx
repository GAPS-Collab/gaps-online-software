#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/complex.h>
#include <pybind11/functional.h>
#include <pybind11/chrono.h>

#include "pybind11_protobuf/native_proto_caster.h"
#include <vector>

#include "WaveGAPS.h"

#include "waveform.h"

//using namespace GAPS;

namespace py = pybind11;

double get_t0(double tA, double tB)
{
    double length =  180;
    double C_LIGHT_PADDLE = 15.5; //cm/ns
    double t0 = 0.5*(tA + tB - (length/(C_LIGHT_PADDLE)));
    return t0;
}

double dist_from_A(double tA, double t0)
{
    double C_LIGHT_PADDLE = 15.5; //cm/ns
    double dA  = (tA - t0)*C_LIGHT_PADDLE;
    return dA;
}

gaps::TofHit waveforms_to_hit(std::vector<double> tA,
                              std::vector<double> wfA,
                              std::vector<double> tB,
                              std::vector<double> wfB)
{
    const double CHARGE_A_MIN = 5.0;
    const double CHARGE_B_MIN = 5.0;

    // the third argument is the channel, but we 
    // are not using that information anyway, so just
    // choose 0 and 1 here as placeholders
    Waveform* wf_a = new Waveform(wfA.data(), tA.data(), 0, 0);
    Waveform* wf_b = new Waveform(wfB.data(), tB.data(), 1, 0);
    std::vector<Waveform*> paddle_waveforms = {wf_a, wf_b};
    for (auto &wf : paddle_waveforms) {
        wf->SetThreshold(10);
        wf->SetCFDSFraction(0.2);
        wf->SetPedBegin(10); // 10-100
        wf->SetPedRange(50);
        wf->CalcPedestalRange();
        wf->SubtractPedestal();
    }
    gaps::TofHit hit;
    hit.set_chargea(1.0*(wf_a->Integrate(270,   70)));
    hit.set_chargeb(1.0*(wf_b->Integrate(270,   70)));
    hit.set_hit_mask(0);
    wf_a->FindPeaks(270, 70);
    if ((wf_a->GetNumPeaks() > 0) && (hit.chargea() > CHARGE_A_MIN))
      {   // then we do cfd
         wf_a->FindTdc(0, CONSTANT);       // Constant threshold
         hit.set_t_at_thra(wf_a->GetTdcs(0));       
         wf_a->FindTdc(0, CFD_SIMPLE);
         hit.set_t_at_cfda(wf_a->GetTdcs(0));
         hit.set_hit_mask(1);
      } 

    wf_b->FindPeaks(270, 70);
    if ((wf_b->GetNumPeaks() > 0) && (hit.chargeb() > CHARGE_B_MIN))
      {   // then we do cfd
         wf_b->FindTdc(0, CONSTANT);       // Constant threshold
         hit.set_t_at_thrb(wf_b->GetTdcs(0));       
         wf_b->FindTdc(0, CFD_SIMPLE);
         hit.set_t_at_cfdb(wf_b->GetTdcs(0));
         hit.set_hit_mask(hit.hit_mask() + 2);
      } 
    
    hit.set_peak_heighta(1.0*wf_a->GetPeakValue(270, 70));
    hit.set_peak_heightb(1.0*wf_b->GetPeakValue(270, 70)); 

    double t0_cfd = get_t0(hit.t_at_cfda(), hit.t_at_cfdb());
    double t0_thr = get_t0(hit.t_at_thra(), hit.t_at_thrb());
    
    hit.set_t0(t0_cfd);
    hit.set_t0_cfd(t0_cfd);
    hit.set_t0_thr(t0_thr);
    hit.set_d_longitude(dist_from_A(hit.t_at_cfda(), t0_cfd)); 

    for (auto &wf : paddle_waveforms) {
        delete wf;
     }
     return hit;
}

PYBIND11_MODULE(pb_dataclasses, m) {
    pybind11_protobuf::ImportNativeProtoCasters();
    m.doc() = "Pybindings for UCLA waveform library";
    m.def("waveforms_to_hit" , waveforms_to_hit);
}

