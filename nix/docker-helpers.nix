# SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
#
# SPDX-License-Identifier: MIT

{ pkgs, ... }:
{
  mkWorker =
    let
      user = "worker";
      group = "worker";
      uid = "1000";
      gid = "1000";
    in
    (pkgs.runCommand "mkUser" { } ''
      mkdir -p $out/etc
      echo "${user}:x:${uid}:${gid}::" > $out/etc/passwd
      echo "${user}:!x:::::::" > $out/etc/shadow
      echo "root:x:0:0::" >> $out/etc/passwd
      echo "root:!x:::::::" >> $out/etc/shadow

      echo "${group}:x:${gid}:" > $out/etc/group
      echo "${group}:x::" > $out/etc/gshadow
      echo "root:x:0:" >> $out/etc/group
      echo "root:x::" >> $out/etc/gshadow

      cat <<EOF > $out/etc/doas.conf
      permit nopass worker
      EOF
    '');
  mkHome = pkgs.stdenv.mkDerivation {
    name = "setup-home";
    buildCommand = "mkdir -p $out/home/worker";
  };
  mkTmp = pkgs.stdenv.mkDerivation {
    name = "setup-tmp";
    buildCommand = "mkdir -p $out/tmp";
  };
  mkEnv = pkgs.stdenv.mkDerivation {
    name = "setup-env";
    buildCommand = ''
      mkdir -p $out/usr/bin
      ln -s ${pkgs.coreutils}/bin/env $out/usr/bin/env
    '';
  };
  mkWorkerPerms = path: regex: {
    inherit path regex;
    mode = "0744";
    uid = 1000;
    gid = 1000;
    uname = "worker";
    gname = "worker";
  };
  mkDoasPerms = path: regex: ({
    inherit path regex;
    mode = "4755";
    uid = 0;
    gid = 0;
    uname = "root";
    gname = "root";
  });

  mkDoas = pkgs.stdenv.mkDerivation {
    name = "doas";
    buildCommand = ''
      mkdir -p $out/sbin
      cp -L ${pkgs.doas.override { withPAM = false; }}/bin/doas $out/sbin/
    '';
  };
}
