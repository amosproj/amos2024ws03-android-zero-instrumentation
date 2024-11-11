# SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
#
# SPDX-License-Identifier: MIT

# Adapted from https://github.com/NixOS/nixpkgs/tree/master/nixos/modules/programs/bash

{ pkgs, ... }:
let

  interactiveShellInit = ''
    	  # Provide a nice prompt if the terminal supports it.
        if [ "$TERM" != "dumb" ] || [ -n "$INSIDE_EMACS" ]; then
          PROMPT_COLOR="1;31m"
          ((UID)) && PROMPT_COLOR="1;32m"
          if [ -n "$INSIDE_EMACS" ]; then
            # Emacs term mode doesn't support xterm title escape sequence (\e]0;)
            PS1="\n\[\033[$PROMPT_COLOR\][\u@\h:\w]\\$\[\033[0m\] "
          else
            PS1="\n\[\033[$PROMPT_COLOR\][\[\e]0;\u@\h: \w\a\]\u@\h:\w]\\$\[\033[0m\] "
          fi
          if test "$TERM" = "xterm"; then
            PS1="\[\033]2;\h:\u:\w\007\]$PS1"
          fi
        fi
    
        # Completion
    	  if shopt -q progcomp &>/dev/null; then
          . "${pkgs.bash-completion}/etc/profile.d/bash_completion.sh"
          nullglobStatus=$(shopt -p nullglob)
          shopt -s nullglob
          for m in "/etc/bash_completion.d/"*; do
            . "$m"
          done
          eval "$nullglobStatus"
          unset nullglobStatus p m
        fi
    	'';

  profile = pkgs.writeTextDir "etc/profile" ''
    	  if [ -n "$__ETC_PROFILE_SOURCED" ]; then return; fi
    
        __ETC_PROFILE_SOURCED=1
    		export __ETC_PROFILE_DONE=1

    		if [ -n "''${BASH_VERSION:-}" ]; then
    		    . /etc/bashrc
    		fi
    	'';

  bashrc = pkgs.writeTextDir "etc/bashrc" ''
    	  if [ -n "$__ETC_BASHRC_SOURCED" ] || [ -n "$NOSYSBASHRC" ]; then return; fi
        __ETC_BASHRC_SOURCED=1

        # If the profile was not loaded in a parent process, source
        # it.  But otherwise don't do it because we don't want to
        # clobber overridden values of $PATH, etc.
        if [ -z "$__ETC_PROFILE_DONE" ]; then
            . /etc/profile
        fi

    		if [ -n "$PS1" ]; then
          ${interactiveShellInit}
        fi
    	'';

in
pkgs.symlinkJoin {
  name = "interactive-bash";
  paths = [
    profile
    bashrc
  ];
}
