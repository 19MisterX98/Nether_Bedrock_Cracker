# Nether_Bedrock_Cracker
Cracks nether seeds from bedrock. This works in 1.18 and upwards since bedrock became seed dependent in that release.
The overworld uses a different RNG so the cool math that's used here doesn't apply.

## Usage

First download the binary for your system from [the releases page](https://github.com/19MisterX98/Nether_Bedrock_Cracker/releases/).<br>
Open it and collect bedrock positions from the nether floor or the nether roof.
After gathering data you can either just run the cracker and show seeds in the gui or run it saving found seeds to a file
<br>
<br>
The rarity of bedrock on a certain y-level is proportional to the information that can be gathered from it. Bedrock on y1 is more common than bedrock on y4, and therefore you need more blocks for the same information. Generally you want ~40 blocks of rare bedrock.

## Problems
### PaperMC servers
Paper had a [bug](https://github.com/PaperMC/Paper/pull/8474) in their code for some time which generated bedrock in a slidely different way. Support for the quirk is planned but not done yet


## Building from source
If you don't want to download the binaries on the releases page you can build from source. <br>
On windows you need rust and msvc. The rest of us only needs to install rust <br>

#### MSVC (for Windows users)
MSVC is bundled in visual studio but if you don't want all the bloat you can download it standalone from: <br>
https://visualstudio.microsoft.com/visual-cpp-build-tools/

#### Rust
Rust is pretty easy to install:
https://www.rust-lang.org/tools/install

#### Downloading source code
Now you need the source code. You can get it by clicking the green button above, then downloading the repository as a zip and then unzipping it on your computer. More experienced users may want to clone the repository instead

#### Compiling
To get the final binary you need to run:

    cargo run --release

After that the executable should be at /target/release/bedrock_cracker
