// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.NamedDomainObjectContainer
import org.gradle.api.file.Directory
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.Provider
import org.gradle.api.provider.ProviderFactory
import org.gradle.api.tasks.TaskContainer
import org.gradle.api.tasks.TaskProvider
import org.gradle.jvm.tasks.Jar
import org.gradle.kotlin.dsl.listProperty
import org.gradle.kotlin.dsl.mapProperty
import org.gradle.kotlin.dsl.newInstance
import org.gradle.kotlin.dsl.property
import org.gradle.kotlin.dsl.register

sealed interface JniLibsVariant {
  val subDir: String
  val name: String

  fun filter(configuration: Configuration): Boolean

  data object DesktopDebug : JniLibsVariant {
    override val name: String = this::class.java.simpleName
    override val subDir: String = "desktop/debug"

    override fun filter(configuration: Configuration): Boolean =
        configuration.target is Target.Desktop && configuration.profile is Profile.Debug
  }

  data object DesktopRelease : JniLibsVariant {
    override val name: String = this::class.java.simpleName
    override val subDir: String = "desktop/release"

    override fun filter(configuration: Configuration): Boolean =
        configuration.target is Target.Desktop && configuration.profile is Profile.Release
  }

  data object AndroidDebug : JniLibsVariant {
    override val name: String = this::class.java.simpleName
    override val subDir: String = "android/debug"

    override fun filter(configuration: Configuration): Boolean =
        configuration.target is Target.Android && configuration.profile is Profile.Debug
  }

  data object AndroidRelease : JniLibsVariant {
    override val name: String = this::class.java.simpleName
    override val subDir: String = "android/release"

    override fun filter(configuration: Configuration): Boolean =
        configuration.target is Target.Android && configuration.profile is Profile.Release
  }
}

object JniLibsVariants {
  fun all() =
      setOf(
          JniLibsVariant.DesktopDebug,
          JniLibsVariant.DesktopRelease,
          JniLibsVariant.AndroidDebug,
          JniLibsVariant.AndroidRelease)
}

abstract class CargoExtension(
    private val taskContainer: TaskContainer,
    private val objectFactory: ObjectFactory,
    private val providerFactory: ProviderFactory,
) {
  companion object {
    const val NAME = "cargo"
  }

  val workspaceDirectory: DirectoryProperty = objectFactory.directoryProperty()

  val libraries: NamedDomainObjectContainer<Library> =
      objectFactory.domainObjectContainer(Library::class.java)

  val binaries: NamedDomainObjectContainer<Binary> =
      objectFactory.domainObjectContainer(Binary::class.java)

  val uniffi = objectFactory.newInstance<UniffiSpec>()

  fun uniffi(action: UniffiSpec.() -> Unit) = uniffi.action()

  private val _jniLibs = objectFactory.mapProperty<JniLibsVariant, Provider<Directory>>()
  val jniLibs: Provider<Map<JniLibsVariant, Provider<Directory>>> = _jniLibs

  private val _jniLibsDesktopDebug = objectFactory.property<TaskProvider<Jar>>()
  val jniLibsDesktopDebug: Provider<TaskProvider<Jar>> = _jniLibsDesktopDebug

  private val _jniLibsDesktopRelease = objectFactory.property<TaskProvider<Jar>>()
  val jniLibsDesktopRelease: Provider<TaskProvider<Jar>> = _jniLibsDesktopRelease

  private val _bindgenDirs = objectFactory.mapProperty<String, Provider<Directory>>()
  val bindgenDirs: Provider<Map<String, Provider<Directory>>> = _bindgenDirs

  internal fun configureLibraries() {
    libraries.configureEach { configure(workspaceDirectory) }
  }

  internal fun configureBinaries() {
    binaries.configureEach { configure(workspaceDirectory) }
  }

  internal fun configureJniLibs(baseDir: Provider<Directory>) {
    JniLibsVariants.all().forEach { configureJniLibsVariant(it, baseDir) }
  }

  private fun configureJniLibsVariant(variant: JniLibsVariant, baseDir: Provider<Directory>) {
    val task =
        taskContainer.register<JniLibsTask>("jniLibs${variant.name}") {
          val prop = objectFactory.listProperty<JniLibsInput>()
          libraries.forEach { prop.addAll(it.jniLibsInputs(variant::filter)) }
          fileInputs.addAll(prop)
          outDir.set(baseDir.map { it.dir(variant.subDir) })
        }

    _jniLibs.put(variant, task.flatMap { it.outDir })
  }

  internal fun jniLibsBridge(name: String, variant: JniLibsVariant): TaskProvider<BridgeTask> =
      taskContainer.register<BridgeTask>(name) {
        inputDirectory.set(jniLibs.flatMap { it[variant]!! })
      }

  internal fun jniLibsJar(
      name: String,
      variant: JniLibsVariant,
      action: Jar.() -> Unit
  ): TaskProvider<Jar> =
      taskContainer.register<Jar>(name) {
        from(jniLibs.flatMap { it[variant]!! })
        action()
      }

  internal fun configureDesktopLibsJar(directory: Provider<Directory>) {
    _jniLibsDesktopDebug.set(
        jniLibsJar("desktopDebugLibs", JniLibsVariant.DesktopDebug) {
          archiveBaseName.set("desktopDebugLibs")
          destinationDirectory.set(directory)
        })
    _jniLibsDesktopRelease.set(
        jniLibsJar("desktopReleaseLibs", JniLibsVariant.DesktopRelease) {
          archiveBaseName.set("desktopReleaseLibs")
          destinationDirectory.set(directory)
        })
  }

  internal fun configureBindgenDirs(baseDir: Provider<Directory>) {
    val outDirs =
        providerFactory.provider {
          libraries
              .map {
                val task =
                    taskContainer.register<UniffiBindgenTask>("uniffiBindgen${it.name}") {
                      library.set(
                          it.outputArtifacts.flatMap {
                            it[Configuration(Target.Desktop.Host, Profile.Debug)] ?: throw Exception("need HostDebug Configuration to be enabled for generating bindings")
                          })
                      outDir.set(baseDir.map { it.dir(name) })
                      workspace.set(workspaceDirectory.map { it.asFile })
                      bindgenCli.set(
                          uniffi.uniffiCli.flatMap {
                            it.flatMap {
                              it.outputArtifacts.flatMap {
                                it[Configuration(Target.Desktop.Host, Profile.Release)]!!
                              }
                            }
                          })
                    }
                it.name to task.flatMap { it.outDir }
              }
              .toMap()
        }
    _bindgenDirs.set(outDirs)
    _bindgenDirs.finalizeValueOnRead()
  }
}
