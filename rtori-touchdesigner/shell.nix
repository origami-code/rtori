with import <nixpkgs> {}; if pkgs.stdenv.isDarwin then {
  qpidEnv = stdenvNoCC.mkDerivation {
    name = "rust-xcode_15_2-env";
    buildInputs = [
        darwin.xcode_15_2
        rustup
        cmake
    ];
    # It is critical to set DEVELOPER_DIR otherwise CMake will complain about version being too low "XCode 1.5 not supported"
    shellHook = ''
PATH=${darwin.xcode_15_2}/Contents/Developer/usr/bin:${darwin.xcode_15_2}/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin:$PATH
SDKROOT=${darwin.xcode_15_2}/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
DEVELOPER_DIR=${darwin.xcode_15_2}/Contents/Developer
'';
  };
} else {}
