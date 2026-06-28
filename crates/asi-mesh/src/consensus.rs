/// Distributed consensus for knowledge sharing across mesh nodes.
pub struct MeshConsensus { pub quorum: usize }
impl MeshConsensus {
    pub fn new(quorum: usize) -> Self { Self { quorum } }
    pub fn reach_consensus(&self, votes: &[bool]) -> bool { votes.iter().filter(|&&v| v).count() >= self.quorum }
}
#[test]
fn test_consensus() { let mc = MeshConsensus::new(3); assert!(mc.reach_consensus(&[true, true, true])); assert!(!mc.reach_consensus(&[true, false, false])); }
