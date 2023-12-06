# Formato dati TofPacket
Per quanto riguarda le strutture (sottoparti di TofPacket) le interessanti sono TofHit, TofEvent ma anche RBEvent, che verrà mantenuta e conterrà i "dati raw" (waveform). Le altre due avranno dati "pre-processati." Achim mi ha confermato, come anticipato, che la cosa è documentata nella wiki e viene tenuta aggiornata, ma non dovrebbe più cambiare in modo sostanziale. Io aggiungo, la wiki è meravigliosa, piena di info, ma un po' arzigogolata. Ho trovato informazioni relative alla struttura dati per la fisica nella [seguente pagina](https://gaps1.astro.ucla.edu/wiki/gaps/index.php?title=GAPS_Time_of_Flight#Dataformats).

Seguendo la pagina si trova anche un [riferimento esteso](https://gaps1.astro.ucla.edu/wiki/gaps/index.php?title=TOF_Event_Dataformats) ad un TofPacket che dovrebbe essere interessante, in linea con OMILU-0.7 (voi lavorate su main, basato su questo branch). Da questo documento si intuisce che, nel caso più probabile (specifico come commenti le cose più interessanti di ogni sezione:
```
<TofPacket - PacketType::TofEvent
  <TofEvent {fields} //qualità del pacchetto (forse più sw che fisica?)
    <TofEventHeader {fields}> //alcune prime quantità ricostruite
    <MasterTriggerEvent {fields}> //timestamp, lista delle hit e paddles coinvolte
    Vec<RBEvent> [
      <RBEvent
        <RBEventHeader {fields}> //housekeeping, monitoraggio
        Vec<Vec<u16>> adc [NCHAN:[NWORDS]] //dati raw, waveform
        Vec<u16>    ch9_adc //canale 9 a parte, contenente il seno per la sincronizzazione
        Vec<TofHit> [
          <TofHit {fields}>, //tempo di arrivo, picco, carica, posizione, ... per lato A e B delle paddle
          ...
        ]
      > //fine RBEvent
    ] //fine lista RBEvent
    Vec<RBMissingHit> [
      <RBMissingHit {fields}> //hit segnalate da MTB ma non ricevute da RB
    ]
  >
>
```

In ultimo, al link delle Github pages della repo [gaps-online-software](https://github.com/GAPS-Collab/gaps-online-software). La sezione per voi interessante è quella delle [tof-dataclasses](https://gaps-collab.github.io/gaps-online-software/tof_dataclasses/index.html#). Qui trovate tutto aggiornato all'ultima versione (NIUHI-0.8), ma stiamo lavorando a rendere disponibile anche quella del main. In ogni caso la struttura dati è pressoché identica e i pacchetti pure. I moduli interessanti per voi sono ```events``` e ```packets```, credo.

Spero questa piccola guida sia utile! Se ci fosse qualcosa chiedete pure. Ho ricalcato un po' quello che è scritto già nella wiki ma almeno ho messo i riferimenti tutti in un posto (il solito lavoro di bricolage :D).