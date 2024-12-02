// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import java.io.Serializable

sealed interface Target : Serializable {
  val cmdlineArgs: List<String>
  val targetPath: String

  sealed interface Android : Target {
    data object Amd64 : Android {
      private fun readResolve(): Any = Amd64

      override val cmdlineArgs = listOf<String>("ndk", "--target", "x86_64")
      override val targetPath = "x86_64-linux-android"
    }

    data object Arm64 : Android {
      private fun readResolve(): Any = Arm64

      override val cmdlineArgs = listOf<String>("ndk", "--target", "arm64-v8a")
      override val targetPath = "aarch64-linux-android"
    }
  }

  sealed interface Desktop : Target {
    data object Host : Desktop {
      private fun readResolve(): Any = Host

      override val cmdlineArgs = listOf<String>()
      override val targetPath = ""
    }
  }
}

sealed interface Profile : Serializable {
  val cmdlineArgs: List<String>
  val targetPath: String

  data object Debug : Profile {
    private fun readResolve(): Any = Debug

    override val cmdlineArgs = listOf<String>()
    override val targetPath = "debug"
  }

  data object Release : Profile {
    private fun readResolve(): Any = Release

    override val cmdlineArgs = listOf<String>("--release")
    override val targetPath = "release"
  }
}

object Profiles {
  fun all() = setOf(Profile.Debug, Profile.Release)
}

object Targets {
  fun all() = setOf(Target.Desktop.Host, Target.Android.Amd64, Target.Android.Arm64)
}

object Configurations {
  fun all() =
      Profiles.all().map { profile ->
        Targets.all().map { target -> Configuration(target, profile) }
      }
}

data class Configuration(
    val target: Target,
    val profile: Profile,
)
