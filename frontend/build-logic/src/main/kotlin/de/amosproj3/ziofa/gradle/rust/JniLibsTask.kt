// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.DefaultTask
import java.io.File
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.FileSystemOperations
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.Nested
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import javax.inject.Inject

abstract class JniLibsInput {

  @get:InputFile @get:PathSensitive(PathSensitivity.NAME_ONLY) abstract val file: Property<File>

  @get:Input abstract val target: Property<Target>
}

abstract class JniLibsTask @Inject constructor(
  private var fileSystemOperations: FileSystemOperations
): DefaultTask() {

  @get:Nested abstract val fileInputs: ListProperty<JniLibsInput>

  @get:OutputDirectory abstract val outDir: DirectoryProperty

  @TaskAction
  fun run() {
    val outDir = outDir.get()
    val fileInputs = fileInputs.get()

    fileSystemOperations.copy {
      into(outDir)

      fileInputs.forEach { from(it.file.get()) { into(it.target.get().jniLibSubdir()) } }
    }
  }
}

private fun Target.jniLibSubdir() =
    when (this) {
      is Target.Android.Amd64 -> "x86_64"
      is Target.Android.Arm64 -> "arm64-v8a"
      is Target.Desktop.Host -> "linux-x86-64" // TODO: handle other os
    }
