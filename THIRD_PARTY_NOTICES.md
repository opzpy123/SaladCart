# Third-party notices

## FFmpeg

SaladCart bundles an FFmpeg executable from the `@ffmpeg-installer/ffmpeg` npm package (FFmpeg 4.4 static build) so that local video preview, thumbnail, beat analysis and MP4 export work without a separate installation.

FFmpeg is a separate executable and is licensed under the LGPL/GPL depending on the build configuration. This bundled build is distributed under the terms of the GNU General Public License; FFmpeg source code and build information are available at:

- https://ffmpeg.org/
- https://ffmpeg.org/download.html
- https://github.com/FFmpeg/FFmpeg

The FFmpeg binary is invoked as a separate process; SaladCart does not link FFmpeg libraries into its own code. If you redistribute a build of SaladCart containing FFmpeg, review the applicable GPL/LGPL obligations and provide the corresponding source or written offer as required.

## Other dependencies

Rust crates and npm packages retain their respective licenses. Their source and license texts are available in `Cargo.lock`, `pnpm-lock.yaml`, and the installed package metadata.

For a production release, replace this development notice with a complete generated SBOM and license bundle.
