use crate::{
    manifest_reader::ManifestReader,
    schematic::{App, Board, Soc},
};

impl ManifestReader for Soc {}
impl ManifestReader for Board {}
impl ManifestReader for App {}
