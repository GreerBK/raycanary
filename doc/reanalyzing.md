# Re-analyzing recordings

Every once in a while, RayCanary refines its heuristics to detect more kinds of
suspicious behavior, and to reduce noise from incorrect alerts.

This means that your old green recordings may actually contain data that is now
deemed suspicious, and also old red recordings may become green.

You can re-analyze any old recording inside of RayCanary by clicking on "N
warnings" to expand details, then clicking the "re-analyze" button.

## Analyzing recordings on Desktop

If you have a PCAP or QMDL file but no raycanary, you can analyze it on desktop
using the `raycanary-check` CLI tool. That tool contains the same heuristics as
RayCanary and will also work on traffic data captured with other tools, such as
QCSuper.

Since 0.6.1, `raycanary-check` is included in the release zipfile.

You can build `raycanary-check` from source with the following command:
`cargo build --bin raycanary-check` 

## Usage
```sh
raycanary-check [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>   Path to the PCAP, or QMDL file. If given a directory will 
                        recursively scan all pcap, qmdl, and subdirectories 
  -P, --pcapify       Turn QMDL file into PCAP     
      --show-skipped  Show skipped messages
  -q, --quiet         Print only warnings
  -d, --debug         Print debug info 
  -h, --help          Print help
  -V, --version       Print version
```
### Examples 
`raycanary-check -p ~/Downloads/myfile.qmdl`

`raycanary-check -p ~/Downloads/myfile.pcap`

`raycanary-check -p ~/Downloads #Check all files in downloads`

`raycanary-check -d -p ~/Downloads/myfile.qmdl #run in debug mode`
