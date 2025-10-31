use super::*;

#[test]
fn test_arp_params() {
    let arp = ArpSynth;
    let mut params = SynthParams::default();

    arp.modify_params(&mut params);

    assert!(params.attack < 0.01);
    assert!(params.release < 0.05);
}
