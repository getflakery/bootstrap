{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang
    # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
    llvmPackages.bintools
    rustup
    pkg-config
    pkgs.openssl.dev
    swagger-codegen
    libiconv

    # python3.withPackages (ps: with ps; [ boto3 ])
    openapi-generator-cli
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];
  RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
  # https://github.com/rust-lacng/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-aarch64-darwin/bin/
  '';
  RUST_BACKTRACE = 1;

  # Add precompiled library to rustc search path
  RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
    #
    # add libraries here (e.g. pkgs.libvmi)
    # "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks"

  ]);
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";


  # # Add glibc, clang, glib and other headers to bindgen search path
  # BINDGEN_EXTRA_CLANG_ARGS = 
  # # Includes with normal include path
  # (builtins.map (a: ''-I"${a}/include"'') [
  #   # add dev libraries here (e.g. pkgs.libvmi.dev)
  #   pkgs.glibc.dev 
  # ])
  # # Includes with special directory paths
  # ++ [
  #   ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
  #   ''-I"${pkgs.glib.dev}/include/glib-2.0"''
  #   ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
  # ];

}
