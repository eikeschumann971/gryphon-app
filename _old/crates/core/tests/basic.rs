use gryphon_core::agent::{Agent, AgentUpdate};
use gryphon_core::batch::AgentBatch;

#[test]
fn agent_round_trip_proto() {
    let agent = Agent::new(42, 1.0, -2.0, 0.5);
    let proto: gryphon_core::proto::gryphon::AgentState = (&agent).into();
    let back: Agent = proto.into();
    assert_eq!(agent, back);
}

#[test]
fn batch_round_trip() {
    let agents = (0..5)
        .map(|i| Agent::new(i, i as f64, 0.0, 0.0))
        .collect::<Vec<_>>();
    let batch = AgentBatch::new(7, agents);
    let proto = batch.to_proto();
    let back = AgentBatch::from_proto(proto);
    assert_eq!(batch.sequence, back.sequence);
    assert_eq!(batch.agents.len(), back.agents.len());
}

#[test]
fn agent_advance_updates_position() {
    let mut agent = Agent::new(1, 0.0, 0.0, std::f64::consts::FRAC_PI_2); // heading = 90 deg
    agent.advance(1.0);
    assert!((agent.x - 0.0).abs() < 1e-9);
    assert!((agent.y - 1.0).abs() < 1e-9);
}