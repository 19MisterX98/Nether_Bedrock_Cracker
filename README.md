# Nether_Bedrock_Cracker
Cracks nether seeds from bedrock. This works in 1.18 and upwards since bedrock became seed dependent in that release.
The overworld uses a different RNG so the cool math thats used here doesnt apply.

## Usage

### !!!The windows build is currently broken!!!

First download the binary for your system from [the releases page](https://github.com/19MisterX98/Nether_Bedrock_Cracker/releases/tag/latest).<br>
Then collect bedrock positions from the nether floor or the nether roof. For formatting you can look at this [example file](https://github.com/19MisterX98/Nether_Bedrock_Cracker/blob/master/example_coords.txt). (Currently I dont support both at the same time but that is planned)
<br>
<br>
The rarity of bedrock on a certain y-level is proportional to the information that can be gathered from it. Bedrock on y1 is more common than bedrock on y4 and therefore you need more blocks for the same information. Generally you want ~40 blocks of rare bedrock.

#### running
After you have your coords file and your bedrock_cracker binary its time to crack! <br>
Open a terminal in the same folder and run:<br>

Windows:

    bedrock_cracker-windows.exe -- coords.txt
    
Linux:

    ./bedrock_cracker-linux -- coords.txt
    
Mac:

    ./bedrock_cracker-mac -- coords.txt


## Problems
### PaperMC servers
Paper had a [bug](https://github.com/PaperMC/Paper/pull/8474) in their code for some time which generated bedrock in a slidely different way. Support for the quirk is planned but not done yet


## Building from source
If you dont want to download the binaries on the releases page you can build from source. <br>
On windows you need rust and msvc. The rest of us only needs to install rust <br>

#### MSVC (for windows users)
MSVC is bundled in visual studio but if you dont want all the bloat you can download it standalone from: <br>
https://visualstudio.microsoft.com/visual-cpp-build-tools/

#### Rust
Rust is pretty easy to install:
https://www.rust-lang.org/tools/install

#### Downloading source code
Now you need the source code. You can get it by clicking the green button above, then downloading the repository as a zip and then unzipping it on your computer. More experienced users may want to clone the repository instead

#### Compiling
To get the final binary you need to run:

    cargo run -- release
    
After that the executable should be at /target/release/bedrock_cracker
