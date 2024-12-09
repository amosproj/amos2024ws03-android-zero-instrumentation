// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import java.io.File
import org.gradle.api.Named
import org.gradle.api.file.Directory
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Provider
import org.gradle.api.provider.ProviderFactory
import org.gradle.api.tasks.TaskContainer
import org.gradle.kotlin.dsl.listProperty
import org.gradle.kotlin.dsl.newInstance
import org.gradle.kotlin.dsl.register
import org.gradle.kotlin.dsl.setProperty
import org.gradle.kotlin.dsl.property
import org.gradle.kotlin.dsl.mapProperty

abstract class Library(
    taskContainer: TaskContainer,
    objectFactory: ObjectFactory,
    providerFactory: ProviderFactory
) : CargoBuild(taskContainer, objectFactory, providerFactory) {
  init {
    artifactName.set(packageName.map { "lib$it.so" })
  }

  override val taskName = "cargoBuildLibrary"

  internal fun jniLibsInputs(filter: (Configuration) -> Boolean): ListProperty<JniLibsInput> {
    val listProperty = objectFactory.listProperty<JniLibsInput>()
    val props =
        outputArtifacts.map { artifacts ->
          artifacts
              .filter { artifact -> filter(artifact.key) }
              .map { artifact ->
                objectFactory.newInstance<JniLibsInput>().apply {
                  target.set(artifact.key.target)
                  file.set(artifact.value)
                }
              }
        }
    listProperty.set(props)
    return listProperty
  }
}

abstract class Binary(
    taskContainer: TaskContainer,
    objectFactory: ObjectFactory,
    providerFactory: ProviderFactory
) : CargoBuild(taskContainer, objectFactory, providerFactory) {
  init {
    artifactName.set(packageName.map { it })
  }

  override val taskName = "cargoBuildBinary"
}

abstract class CargoBuild(
    protected val taskContainer: TaskContainer,
    protected val objectFactory: ObjectFactory,
    protected val providerFactory: ProviderFactory
) : Named {
  protected val artifactName = objectFactory.property<String>()

  abstract val taskName: String

  val targets = objectFactory.setProperty<Target>().convention(Targets.all())

  val profiles = objectFactory.setProperty<Profile>().convention(Profiles.all())

  val packageName = objectFactory.property<String>().convention(providerFactory.provider { name })

  val features = objectFactory.listProperty<String>()

  val extraArgs = objectFactory.listProperty<String>()

  private val _outputArtifacts =
      objectFactory.mapProperty<Configuration, Provider<File>>().apply { finalizeValueOnRead() }
  val outputArtifacts = _outputArtifacts

  internal fun configure(workspaceDirectory: Provider<Directory>) {
    targets.finalizeValueOnRead()
    profiles.finalizeValueOnRead()

    val artifacts =
        providerFactory.provider {
          val realizedTargets = targets.get()
          val realizedProfiles = profiles.get()

          val prop = objectFactory.mapProperty<Configuration, Provider<File>>()
          realizedTargets
              .flatMap { target ->
                realizedProfiles.map { profile -> Configuration(target, profile) }
              }
              .associateWith { config ->
                objectFactory
                    .property<Provider<File>>()
                    .value(
                        providerFactory.provider { configureVariant(config, workspaceDirectory) })
                    .apply { finalizeValueOnRead() }
                    .flatMap { it }
              }
        }
    _outputArtifacts.set(artifacts)
  }

  private fun configureVariant(config: Configuration, dir: Provider<Directory>): Provider<File> {
    val task =
        taskContainer.register<CargoBuildTask>("$taskName$name$config") { // TODO nicer name
          workspaceDirectory.set(dir.map { it.asFile })
          outputFile.set(
              workspaceDirectory.zip(artifactName) { ws, pn ->
                File("$ws/target/${config.target.targetPath}/${config.profile.targetPath}/$pn")
              })
          cmdlineArgs.addAll(config.target.cmdlineArgs)
          cmdlineArgs.addAll("build")
          cmdlineArgs.add("--package")
          cmdlineArgs.add(packageName)
          cmdlineArgs.addAll(
              features.map {
                it.joinToString(",").takeIf { it.isNotBlank() }?.let { listOf("--features", it) }
                    ?: listOf()
              })
          cmdlineArgs.addAll(config.profile.cmdlineArgs)
          cmdlineArgs.addAll(extraArgs)
        }
    return task.flatMap {
      it.outputFile
    }
  }
}
