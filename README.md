# Nether Bedrock Cracker

Cracks nether seeds from bedrock. This works in versions 1.18 and above since bedrock became seed dependent in that release. The overworld uses a different RNG, so the calculations used here are not applicable.

## Usage

To use the Nether Bedrock Cracker, follow these steps:

1. Download the application for your system from [the releases page](https://github.com/19MisterX98/Nether_Bedrock_Cracker/releases/).
2. Open the downloaded application.
3. Collect bedrock positions from both the nether floor and the nether roof.
    - Collect data from both the floor and the ceiling. Gathering data from only one side will result in less informative results.
    - Focus on collecting bedrock data on y-level 4 or y-level 123, as bedrock is rarer in those layers, providing more valuable information per block.

After gathering the required data, you have two options:

1. Run the cracker and view the cracked seeds in the graphical user interface (GUI).
2. Run the cracker and save the found seeds to a file.

## Known Issues

### PaperMC Servers
PaperMC had a [bug](https://github.com/PaperMC/Paper/pull/8474) in their code that caused slight differences in bedrock generation. If you encounter a situation where your bedrock is shaped in pillars, similar to the bug report, switch the crack mode in the GUI from "Normal" to "Paper < 1.19.2-213."

### Old Nether
If your nether was generated in a version older than 1.18, you can use the [fungus cracker](https://youtu.be/HKjwgofhKs4) of [SeedcrackerX](https://github.com/19MisterX98/SeedcrackerX) to crack the seeds.

## Building from Source

If you prefer to build the application from source instead of downloading it from the releases page, follow these steps:

### Prerequisites

#### Windows
You will need to install Microsoft Visual C++.
MSVC is bundled with Visual Studio, but if you prefer a standalone installation, you can download it from:
https://visualstudio.microsoft.com/visual-cpp-build-tools/

### Building

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Download the repository.
    - Click the green button above to download the repository as a zip.
    - Unzip the downloaded file on your computer. Advanced users may prefer cloning the repository instead.
3. Open a terminal in the unzipped directory.
4. Run the command `cargo build --release`.
5. The executable should now be located at `/target/release/bedrock_cracker`.
