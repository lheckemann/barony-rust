with import <nixpkgs> {};
runCommand "bars" {
  buildInputs = [
    (rustChannelOf { date = "2018-04-10"; channel = "nightly";} ).rust
    gcc
    alsaLib
    pkgconfig
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath ([ libGL ] ++ (with xorg; [
    libX11 libXcursor libXrandr libXi
  ]));
  shellHook = ''
    export PATH=$HOME/.cargo/bin:$PATH
  '';
} ''echo "This is a shell environment, silly!"''
