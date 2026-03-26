# SSBU Find Player
## Description
Determine if you were player 1 (left side) or 2 (right side) in given SSBU recordings.

(warning: not very useful and very hacky, this was just built to play around with rust and image processing)

## Usage
Uses ffmpeg to extract a frame from a clip and does template matching with a given name image.

```
cargo run <ffmpeg path> <dir with clips> <dir to save frames> <name template>
```

## Example
Given an extracted frame from a clip `2021090420284001-0E7DF678130F4F0FA2C88AE72B47AFDF.mp4`:

![Example extracted frame](/example/player1.jpg)

and an image with the name tag:

![Example image of name tag](/example/name.jpg)

the command
```
cargo run ../ffmpeg/bin /clips /frames /example/name.jpg
```

produces
```
In: \clips\2021090420284001-0E7DF678130F4F0FA2C88AE72B47AFDF.mp4
Out: \frames\2021090420284001-0E7DF678130F4F0FA2C88AE72B47AFDF.jpg
Player 1 for \clips\2021090420284001-0E7DF678130F4F0FA2C88AE72B47AFDF.mp4
```


## Notes

Only useful for clips directly recorded on Nintendo Switch which outputs as 1280x720. Also some issues with an incorrect directory structure as input.



