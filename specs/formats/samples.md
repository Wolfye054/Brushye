# Format samples

Registry of the sample files the format study uses (DECISIONS D-020). The files live outside
the repo (`~/brushye-samples/`). The full steps for each sample are in its `registry.md` there.

## .sut (CSP 1.13.2, automated-Wine, 2026-11-13; batch 2 is S-63..S-116)
Each sample is a new custom sub tool (Pen group, Direct draw / Pen), set to S-01's settings
plus the listed change.

| ID | File | SHA-256 | Content |
|---|---|---|---|
| S-01 | `S-01_base.sut` | `c32aba0361e0ca17e8ca6140af1c763fcd371fcc7372eaa36eb9336445951b72` | New custom sub tool "Base" (Pen group), defaults + Brush Size 10 |
| S-02 | `S-02_base_again.sut` | `0742aaf1589c1918df1d397fac2273d5e9c712dbbc06c34efc1090a929382dc8` | S-01's sub tool exported again, no changes |
| S-03 | `S-03_renamed.sut` | `3ea8769c08a913321c3d13e395dd541461c8f7c383afd89a11a83d493b172e8d` | S-01's sub tool renamed "Base Renamed" |
| S-04 | `S-04_icon.sut` | `47104a10c50874e047cf3c3564d777df2531e3d81b874519d35373c664cec6bd` | S-01's sub tool (name back to "Base") with icon background colour #336699 |
| S-10 | `S-10_size_1.sut` | `82f3fc5d21c4108de41017437ddab37d3975e91af6827934c919a5d0fd660a13` | Brush Size 1 |
| S-11 | `S-11_size_2.5.sut` | `9db51300396449d9a81cea0db258eef32e6835824b10a07628489459d58b05d7` | Brush Size 2.5 |
| S-12 | `S-12_size_50.sut` | `bd1daf10c243861ebc3d73a2cd8f396a1685dd30ae4b8779164687bf509ffa00` | Brush Size 50 |
| S-13 | `S-13_size_500.sut` | `f57f41c376522d3ecbcb183db849343ea8cf72803d2766cb3a01af8165edb3b4` | Brush Size 500 |
| S-14 | `S-14_size_2000.sut` | `bef65c35fd2c20a7252d33398284d468cecdb4e1919c6510dbde9646f852259c` | Brush Size 2000 |
| S-15 | `S-15_opacity_1.sut` | `b2a054847fb89b47682f237085dc3b33204c8a768257e16ca7c0d9f06b93ac7a` | Opacity 0 requested; CSP clamped to 1 |
| S-16 | `S-16_opacity_37.sut` | `286215de702be6ae0eb823fae2448c727aaf8383ba75821aa5e60f730c05fae6` | Opacity 37 |
| S-17 | `S-17_opacity_99.sut` | `ef92aa2e635f7cba3662a11e1595cf404e89a0bbbf18fbc13f1188fed42a4785` | Opacity 99 |
| S-18 | `S-18_aa_none.sut` | `b275e3392f2906c0dcaefc2621258da09cc4b3ff687fb8b492466fc94a1a38e1` | Anti-aliasing None |
| S-19 | `S-19_aa_weak.sut` | `5e785ea193c701330c9d23a3d4892ac6f239ac5f4c62e45108381f2ac9af8759` | Anti-aliasing Weak |
| S-20 | `S-20_aa_strong.sut` | `0f682b37ec94e79d893d03cbfac219bbfd7bb726e7367a0ff809e4d8a937e23f` | Anti-aliasing Strong |
| S-22 | `S-22_hardness_1.sut` | `3a8b862fc7d54562c750e2c91e4674129f4a26a444a53bafc1d556759817a337` | Hardness min (typed 0, CSP clamped to 1) |
| S-23 | `S-23_hardness_25.sut` | `f30d4f2480fadb95524329e4541fc9b3d92024b3c11234c2636c645ee4e08f6c` | Hardness 25 |
| S-24 | `S-24_hardness_75.sut` | `239bfb80ad067e31ad01b2ea5f53c8cd92b8c4bd7aa4a2feac2fbad9677158df` | Hardness 75 |
| S-25 | `S-25_hardness_100.sut` | `921163cdd3202f8625e903af9106ef6fd312a59efec04a906302376f5fc4c84f` | Hardness max = 100 (same as default; control sample) |
| S-26 | `S-26_thickness_10.sut` | `467eba121757bb9c02b232afe94700c7b63a647b24a32e601dede5ee3e88a40c` | Thickness 10 |
| S-27 | `S-27_thickness_50.sut` | `7bffdeb45915b625e643b9047df4ed7b92afd02c2825533caeb71f9fea50f125` | Thickness 50 |
| S-28 | `S-28_thickness_100_vertical.sut` | `2518fbf4a6d362d873445aa9a92d8d52b63899e14605251754f627311b912a5c` | Thickness 100 + Direction Vertical |
| S-29 | `S-29_angle_0.sut` | `473d6988057d1e103f7b6cc37c9916c19f50d5cb811c2671929b760f9bf78b20` | Angle 0 (= default; control sample) |
| S-30 | `S-30_angle_45.sut` | `0817daa9fa90f169ffaebffc4708bb95cae2adee2859d0caa48939726abea6ed` | Angle 45 |
| S-31 | `S-31_angle_359.sut` | `d7314acffdb057213aeaac8e6c35ff4616e7070d0c3598d90e8a1235c472ce83` | Angle 359 |
| S-32 | `S-32_gap_narrow.sut` | `97d0ab5b7919fab7077ca18644674fe202be338cb06fc3f0a79fbf14b558e6e2` | Gap Narrow |
| S-33 | `S-33_gap_normal.sut` | `70fb0827531f4a92d087c8a1f62f431e04e79bc0d4a07304375ef0803c6a261b` | Gap Normal (= default; control sample) |
| S-34 | `S-34_gap_wide.sut` | `b584e362bd4c7db1bea836f7bbe5d092611163eb2ae2875d3b8872d4b50762a6` | Gap Wide |
| S-35 | `S-35_gap_fixed_7.sut` | `28860ab14b95ecdc604900050cd02adecc2d6ebda3bc9601c60b87a83b4cd4fa` | Gap Fixed, value 7 |
| S-36 | `S-36_density_50.sut` | `9da0ef29d3f00352792729c1943e897c72090cd86ca97eafb0ec927ab0e10507` | Brush density 50 |
| S-37 | `S-37_pressure_size_min20.sut` | `cd30411543160bf50b97a1e3af137fc2706587d80b8e9e33084c5156ef498194` | Pressure → size, minimum 20 |
| S-38 | `S-38_pressure_size_min20_scurve.sut` | `2cbcfd6ca43155dc28256b796439be14ff416ddcfa1e2b6a12aeba5cc80b45fa` | S-37 + custom pressure curve (S-curve) |
| S-39 | `S-39_pressure_opacity_min0.sut` | `f9b15ea00ab0cbf257e72974cca2173d9112151839133a6c9aba9a55149a02e6` | Pressure → opacity ON, min 0 |
| S-40 | `S-40_pressure_density.sut` | `ca04248d0c51dd06b060277e1e76e7bc9146a62503ab3cbfb74c5e9a560eb4f2` | Pressure → brush density ON |
| S-41 | `S-41_velocity_size.sut` | `8bfd0c92784f1bed426e2be21d80182cd6d1a67aa7a4f18111e36c2f67a26e6a` | Velocity → size ON |
| S-42 | `S-42_random_size.sut` | `ba4186063cd656457fb292c60a1d2023bd4f29bcf87d0c9ba863391760ba52a0` | Random → size ON |
| S-43 | `S-43_tilt_thickness.sut` | `13d79c7a77706993eb0bf200e20fc9e193f672e0e64e7e2aaef9bc31975469ae` | Tilt → thickness ON |
| S-44 | `S-44_blend_multiply.sut` | `f09bf2153aba33180eadd538f56f8785512430f47af7af74aea487da3d0dd678` | Ink blending mode Multiply |
| S-45 | `S-45_blend_compare_density.sut` | `028cf2f758e1ab482b047a42fbcdb3e03d13b6ac2dff6c4f64295452c6709b6a` | Blending mode Compare density |
| S-46 | `S-46_blend_background.sut` | `54e7cea7be1704bc13e4be38aa3157aeded590f1ef9c3ee6efee957e66fb804d` | Blending mode Background |
| S-47 | `S-47_blend_replace_alpha.sut` | `93f2cac362458f8e5ff9ffa8d4acde8fadb5ce3c09aefea4a88172eb3a4fe745` | Blending mode Replace alpha |
| S-48 | `S-48_mix_blend_60_40_30.sut` | `0434cbd2353100db85dd0241e01b24c058447eef1db6eb0e0b681381228bd8f4` | Color mixing Blend, paint amount 60, density 40, stretch 30 |
| S-49 | `S-49_mix_running_color.sut` | `f4fad9084086810df1407b5c659dc850ccb8f6a0b7ed0ca01d2196c5e0f5fa7c` | Color mixing Running color |
| S-50 | `S-50_mix_smear.sut` | `7f98c12d32737b72c2d19d3868e57d3249292f5fbd988898cc377ad5c0a35361` | Color mixing Smear |
| S-51 | `S-51_stabilization_0.sut` | `c58089287872c119230aa2a9f4496ca1e9f50768bda743612ebdacb901877bdf` | Stabilization 0 |
| S-52 | `S-52_stabilization_30.sut` | `e17cdf057a773768602ff7c3b9e6fcc4432cb2250a8f63368990ba166d3109d1` | Stabilization 30 |
| S-53 | `S-53_adjust_by_speed_off.sut` | `e6df6ef864d0677136575697a7ffc822dc20135730785a6c24a1b77e92cf8230` | Adjust by speed OFF |
| S-54 | `S-54_start_end_size_20_40.sut` | `9d0b7b9a808b5af6f7ccb9992814b21fc8fa07ab66391f5a654e9883e7304da9` | Starting and ending: Brush Size, start 20, end 40 |
| S-55 | `S-55_texture_noise_50_scale120.sut` | `b3c8cf359eccb28190d34bbdf21a765dfd7e034b69d2988817dc05c6c8f9d7fa` | Texture: own noise image, density 50, scale 120 % |
| S-56 | `S-56_spray_10_20_50.sut` | `ea4c9d5cd8033f3797b3db0e6776712cba3e9e26fd9aaefc323dfa310b42bc97` | Spraying effect on, particle size 10, density 20, deviation 50 |
| S-57 | `S-57_watercolor_5_50_50_blur3.sut` | `3d2e6d91f874e8f005a4c6965c44a8428b5ea208902542c1446bd0a7935d3525` | Watercolor edge on, width 5, opacity 50, darkness 50, blur 3 |
| S-58 | `S-58_jitter_hsl_10.sut` | `3f574f63084f3414cc0ed515347bd87013c04fb44104b018ac24fb241d234089` | Colour jitter on, H/S/L 10 |
| S-59 | `S-59_material_tip_star.sut` | `28d5f25a2d586b2a6f84d5406184dce5ad7b60dea637fb854d720525f5b4a4a3` | Material tip: own 64×64 star |
| S-60 | `S-60_material_tips_star_ring_random.sut` | `96363be8939b11dedcb1af3c79e67c9fb86bb021443af79e262a466d847c3b32` | Two material tips (star, ring) + Repeat method Random |
| S-61 | `S-61_dual_brush_star.sut` | `c7af74d97e61de6188fa8455793f70f16f9e8d318d30d55039aa9d692435f76e` | Dual brush on with own star tip |
| S-62 | `S-62_ribbon_star_tip.sut` | `b214d8e142f27c1faa1367604c9b23dde497894046743b7f82a4a1616a194473` | Ribbon on (needs a material tip: star tip added too) |
| S-63 | `S-63_hardness_seg1.sut` | `0331c4aeb57878fce6f594e82c392fd231f3440e6f7642e55b528e793dffc864` | Hardness by clicking bar segment 1 of 5 → number box shows **1** |
| S-64 | `S-64_hardness_seg2.sut` | `ea5ac732c1983ebf2e1226cf8cd831e6b774cda921139323decee2958c97251c` | Hardness by clicking bar segment 2 of 5 → number box shows **30** |
| S-65 | `S-65_hardness_seg3.sut` | `cb5a6c72ca156b53ea4a6c3738219b9e48d5b03bead2350917dead0dc650b51b` | Hardness by clicking bar segment 3 of 5 → number box shows **60** |
| S-66 | `S-66_hardness_seg4.sut` | `5e6f06fa63d6e2818a7dce162e20c09d03e791b3795d891522cee6eb432411bf` | Hardness by clicking bar segment 4 of 5 → number box shows **95** |
| S-67 | `S-67_hardness_seg5.sut` | `5d041ad6853798a26884da4322ea1494307672bd5d967dde7d78be5119bb8566` | Hardness by clicking bar segment 5 of 5 → number box shows **100** |
| S-68 | `S-68_hardness_50.sut` | `c3dc18886f092547d9cdb0c2beb6c7468b0e6b3c530043c0badf597d4cdc2835` | Hardness typed 50 → bar lights **3 of 5** segments |
| S-70 | `S-70_blend_darken.sut` | `1deef2b218af2275f39e087ff5f3b84e4549489c0c77ff8acc82cc07de777fb2` | Ink blending mode Darken |
| S-71 | `S-71_blend_color_burn.sut` | `a52e794528d430a3a7194513e1f61ae2e7d4fa3d74e3039f86568dc7510a60a7` | Ink blending mode Color burn |
| S-72 | `S-72_blend_linear_burn.sut` | `7d15c6478b7c4458dea73e08d47a23fce703c6c931423dfe9c704b1d28b37b00` | Ink blending mode Linear burn |
| S-73 | `S-73_blend_black_burn.sut` | `8f62f9b9f5818473cd8fbd465fa92a866a870d35394581d3fbb0b616c10242c9` | Ink blending mode Black burn |
| S-74 | `S-74_blend_subtract.sut` | `ae2759f301316afd4e3372e546b732248c9d8f841a4d47cdaa39c193767baea1` | Ink blending mode Subtract |
| S-75 | `S-75_blend_lighten.sut` | `0a2f674512fd3d36f0617a77fc5bb42965ae9c7e5aa36bfa5d1872e2436f269d` | Ink blending mode Lighten |
| S-76 | `S-76_blend_screen.sut` | `e551d35671f0ed7ef92f8a5fd275c7246bd6e49972598dbe7f6bbee2b289bb43` | Ink blending mode Screen |
| S-77 | `S-77_blend_color_dodge.sut` | `608946fe9b3a3a4d077f9156b26ad2f60189268fafac2f4eb6acbd9ef1dade80` | Ink blending mode Color dodge |
| S-78 | `S-78_blend_glow_dodge.sut` | `d4501a3ecf0f2d75f61d55172997d602650ff911af2123f051ce06f92bc9579f` | Ink blending mode Glow dodge |
| S-79 | `S-79_blend_white_burn.sut` | `a93058047bce7dc337f57b4728f13d0e37be5e6fe13cef11c053c07eb8e8dded` | Ink blending mode White burn |
| S-80 | `S-80_blend_add.sut` | `bab7769976723773fc92c91297fa611d49acc1bae1ddcdafe7bf3ad80c0b3039` | Ink blending mode Add |
| S-81 | `S-81_blend_add_glow.sut` | `61c9139349ed03f25069dc50add94b99d1378842b6b468336fa5419d3c74a574` | Ink blending mode Add (Glow) |
| S-82 | `S-82_blend_overlay.sut` | `120df316ba0028a0ceab5ad5b1f5f1cda73cd0282462d95890c607779b97c5e4` | Ink blending mode Overlay |
| S-83 | `S-83_blend_soft_light.sut` | `a5479e63f9742901cd063b25230df0aee89692970ac0a23db2251c889e458424` | Ink blending mode Soft light |
| S-84 | `S-84_blend_hard_light.sut` | `38f4f851dcb82b0cacc418e98ad75770c05afe12bbb82d28da118208e0b031a7` | Ink blending mode Hard light |
| S-85 | `S-85_blend_difference.sut` | `82c95e111789136845c88036d176704800d857cc89c16046a605c29521762dce` | Ink blending mode Difference |
| S-86 | `S-86_blend_erase.sut` | `cec775d53256199ddaf29178a95b67bc235227607925d8207b2450ea07d2b00f` | Ink blending mode Erase |
| S-87 | `S-87_blend_erase_compare.sut` | `ab00dfaef8086a6842cfd53b40af0dc0c1e3e311792e6575e61c1da61c372944` | Ink blending mode Erase (compare) |
| S-88 | `S-88_blend_vivid_light.sut` | `f1c2dcaaae8cc0346e19e4c7f40e9f5630ef7a7d6b542fe0d620e4118f612fff` | Ink blending mode Vivid light |
| S-89 | `S-89_blend_linear_light.sut` | `dca7d66c2379ba05963f0892b8468b0042cb3385b9a134ee0f5921547d51da3a` | Ink blending mode Linear light |
| S-90 | `S-90_blend_pin_light.sut` | `2d256bf959a0b1e1decf7acbd95019c747ea726e935f8cda590efee00b7308b5` | Ink blending mode Pin light |
| S-91 | `S-91_blend_hard_mix.sut` | `dee7a51b73269a1398fd5c74d91a5e00202e0529a604a8fa1ddbf3b5d2361d40` | Ink blending mode Hard mix |
| S-92 | `S-92_blend_exclusion.sut` | `8559ee9a6c098bf90319e91e9420172e745499b1eb6e246db3e36b24cb280eff` | Ink blending mode Exclusion |
| S-93 | `S-93_blend_darker_color.sut` | `0a9c63ccdb57600610ffad7d6622f25150023c84113dada66c104a7f32354e9a` | Ink blending mode Darker color |
| S-94 | `S-94_blend_lighter_color.sut` | `b8ed2bfc8ec4d6511756ae7acfe7934d5211f53191e19202d69e6741a823839b` | Ink blending mode Lighter color |
| S-95 | `S-95_blend_divide.sut` | `9259d2e542c873fcb7a800b597d3fa745ed21029938f18699fad3b29de5ac3d5` | Ink blending mode Divide |
| S-96 | `S-96_blend_hue.sut` | `86581f65e24dafc99187a78325d0819e3da3e39fbdb8c56bde1477d90a4f2acb` | Ink blending mode Hue |
| S-97 | `S-97_blend_saturation.sut` | `0ca0cef0dfd250366594bf1c6a03f9c486ab09b3c516a1e7a80ec1cda55ccd53` | Ink blending mode Saturation |
| S-98 | `S-98_blend_color.sut` | `b767228a196e5e53972107436c060381b6b899ded55b5920bcf19df038012087` | Ink blending mode Color |
| S-99 | `S-99_blend_brightness.sut` | `ae9da806bf5d4f2a4287b8823b27a49d3ef614f3591ac22ed66141f9ede5aced` | Ink blending mode Brightness |
| S-110 | `S-110_angle_direction_of_line.sut` | `b75f1f7008403b5fbaa94d71a75958f65ee5153ef61ef43d1bb5e74c86b6dc49` | Angle dynamics: Direction of line |
| S-111 | `S-111_density_adjust_by_gap.sut` | `16c4e66e769dafe6b363e8ab972e093f3569c073ce2fa7e6fe69ff77cc7389ab` | Adjust brush density by gap ON |
| S-113 | `S-113_size_on_screen.sut` | `5903bd053ab84e38e9a0b2c5a1e1a52370a9def0e8b860a2473613d2f81ab5c8` | Specify by size on screen ON |
| S-114 | `S-114_at_least_1px.sut` | `2ac0ad241d8418451c5cb2033e42605ee77dfc8bc7ffefec24e4ca23c850fd1b` | At least 1 pixel ON |
| S-115 | `S-115_eraser_group_base.sut` | `7f27939715113fa766049cc27bd5ef09b0edfea6f21cdfabaa72a803ca154692` | New custom sub tool in the Eraser tool's group, size 10 |
| S-116 | `S-116_airbrush_group_base.sut` | `fa7b27b1889278e9e108f7175550bfc0af7053ee3de87976659dc7814b1c09fb` | New custom sub tool in the Airbrush tool's group, size 10 |
