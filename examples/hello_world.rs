use bevy::prelude::*;
use leafwing_abilities::HelloWorldPlugin;

fn main() {
    App::build().add_plugin(HelloWorldPlugin).run();
}
