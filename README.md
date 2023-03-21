# Nether_Bedrock_Cracker
Cracks nether seeds from bedrock. This works in 1.18 and upwards since bedrock became seed dependent in that release.
The overworld uses a different RNG so the cool math thats used here doesnt apply.

## Usage
### Rust
Install Rust https://www.rust-lang.org/tools/install <br>
On windows rust requires MSVC which normally comes with visual studio.<br> 
That said visual studio is pretty bloated so you're probably better off installing only MSVC from https://visualstudio.microsoft.com/visual-cpp-build-tools/ <br>
After that consider using linux cause there it was just one command

### Repo
Next clone/download the repository and open the main.rs file to configure bedrock positions and thread count

### Running
open your terminal in the repository folder and type:

    cargo run
    
The program should now start and be done in a few minutes. For me it takes ~12 minutes on a single thread
