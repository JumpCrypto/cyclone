# cyclone-msm

Assuming the FPGA image is loaded with `make reset`.

Note that this image `agfi-09bec09a9e2b4d332` has a fix to enable reading out points
without DRAM, compared to the image `agfi-0d25a1d127f1b497f` of the ZPrize submission.

Default SIZE=16.

- `make points SIZE=<SIZE>` generates files `size<SIZE>.beta` and `size<SIZE>.points`.
- `make column SIZE=<SIZE>` calculates a column MSM using these points.
- `make msm SIZE=<SIZE>` calculates a full 16 column MSM using these points.

You can also skip loading points in the column and msm targes:
- `make load SIZE=<SIZE>` generates files `size<SIZE>.beta` and `size<SIZE>.points`.
- `make column-pre SIZE=<SIZE>` calculates a column MSM using these points.
- `make msm-pre SIZE=<SIZE>` calculates a full 16 column MSM using these points.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

