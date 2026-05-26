// Evita que se abra una consola adicional en Windows para builds release.
// NO eliminar — la consola filtra la app de "background app" en Defender.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

fn main() {
    cleartool_lib::run()
}
