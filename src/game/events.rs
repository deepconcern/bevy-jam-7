use bevy::prelude::*;

#[derive(Debug, Event)]
// Minigame is over (transition to results)
pub struct MinigameFinished(pub bool);

#[derive(Debug, Event)]
// Minigame has spawned, so finish transition
pub struct MinigameSpawned;

#[derive(Debug, Event)]
// Minigame is ready to be played
pub struct MinigameStart;

#[derive(Debug, Event)]
// Kicks of transition to minigame
pub struct NewMinigame(pub String);

#[derive(Debug, Event)]
pub struct InterludeStart;

#[derive(Debug, Event)]
// Results are finished, transition to interlude or game over
pub struct ResultsSpawned(pub bool);

#[derive(Debug, Event)]
// Screen is covered, so spawn minigame
pub struct SpawnMinigame(pub String);

#[derive(Debug, Event)]
// Results are shown
pub struct SpawnResults(pub bool);
