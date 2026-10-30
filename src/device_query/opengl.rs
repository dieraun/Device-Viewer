//! OpenGL renderer identity and GPU telemetry.
//!
//! A headless OpenGL context cannot be created portably, so the data layer
//! exposes [`query_from_context`] for the GUI render thread. The autonomous
//! [`query`] reports the section as unavailable instead of failing the pass.

use std::collections::BTreeMap;

use glow::HasContext;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::error::QueryResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenGlInfo {
    pub available: bool,
    pub vendor: Option<String>,
    pub renderer: Option<String>,
    pub version: Option<String>,
    pub shading_language_version: Option<String>,
    pub max_texture_size: Option<i32>,
    pub limits: BTreeMap<String, i32>,
    pub extensions: Vec<String>,
    pub warnings: Vec<String>,
}

/// Reads strings and limits from a live context.
///
/// Must run on the thread that owns the OpenGL context. Kept as the public
/// entry point for the render thread even though the headless collector cannot
/// create a context by itself.
#[allow(dead_code)]
pub fn query_from_context(gl: &glow::Context) -> OpenGlInfo {
    let mut info = OpenGlInfo {
        available: true,
        ..OpenGlInfo::default()
    };

    unsafe {
        info.vendor = Some(gl.get_parameter_string(glow::VENDOR));
        info.renderer = Some(gl.get_parameter_string(glow::RENDERER));
        info.version = Some(gl.get_parameter_string(glow::VERSION));
        info.shading_language_version =
            Some(gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION));
        info.max_texture_size = Some(gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE));
        info.extensions = gl
            .get_parameter_string(glow::EXTENSIONS)
            .split_whitespace()
            .map(str::to_string)
            .collect();

        let limits = [
            ("MAX_TEXTURE_SIZE", glow::MAX_TEXTURE_SIZE),
            ("MAX_CUBE_MAP_TEXTURE_SIZE", glow::MAX_CUBE_MAP_TEXTURE_SIZE),
            ("MAX_RENDERBUFFER_SIZE", glow::MAX_RENDERBUFFER_SIZE),
            ("MAX_VERTEX_ATTRIBS", glow::MAX_VERTEX_ATTRIBS),
            ("MAX_TEXTURE_IMAGE_UNITS", glow::MAX_TEXTURE_IMAGE_UNITS),
            (
                "MAX_VERTEX_TEXTURE_IMAGE_UNITS",
                glow::MAX_VERTEX_TEXTURE_IMAGE_UNITS,
            ),
            (
                "MAX_COMBINED_TEXTURE_IMAGE_UNITS",
                glow::MAX_COMBINED_TEXTURE_IMAGE_UNITS,
            ),
        ];
        for (name, key) in limits {
            info.limits
                .insert(name.to_string(), gl.get_parameter_i32(key));
        }
    }

    info
}

pub async fn query(_cancel: &CancellationToken) -> QueryResult<OpenGlInfo> {
    Ok(OpenGlInfo {
        available: false,
        warnings: vec![
            "OpenGL identity requires a live rendering context; the GUI supplies it on the render thread"
                .to_string(),
        ],
        ..OpenGlInfo::default()
    })
}
