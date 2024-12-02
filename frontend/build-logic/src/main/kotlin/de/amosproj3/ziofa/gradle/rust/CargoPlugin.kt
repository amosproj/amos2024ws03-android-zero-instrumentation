// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import com.android.build.api.variant.LibraryAndroidComponentsExtension
import org.gradle.api.NamedDomainObjectContainer
import org.gradle.api.NamedDomainObjectProvider
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.VersionCatalog
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.api.tasks.Copy
import org.gradle.internal.extensions.stdlib.capitalized
import org.gradle.kotlin.dsl.dependencies
import org.gradle.kotlin.dsl.register

val NamedDomainObjectContainer<Configuration>.implementation:
    NamedDomainObjectProvider<Configuration>
  get() = named("implementation")

val NamedDomainObjectContainer<Configuration>.desktopLibs: NamedDomainObjectProvider<Configuration>
  get() = named("desktopLibs")

val NamedDomainObjectContainer<Configuration>.desktopDebugLibs:
    NamedDomainObjectProvider<Configuration>
  get() = named("desktopDebugLibs")

val NamedDomainObjectContainer<Configuration>.desktopReleaseLibs:
    NamedDomainObjectProvider<Configuration>
  get() = named("desktopReleaseLibs")

internal class VersionCatalogLibs(private val inner: VersionCatalog) : VersionCatalog by inner {
  val jna = findLibrary("jna").get()
  val jnaAar = jna.map { it.apply { artifact { type = "aar" } } }
  val jnaJar = jna.map { it.apply { artifact { type = "jar" } } }
  val coroutinesAndroid = findLibrary("kotlinx.coroutines.android").get()
  val androidLibrary = findPlugin("android.library").get().get().pluginId
  val kotlinAndroid = findPlugin("kotlin.android").get().get().pluginId
}

internal val Project.libs
  get() =
      VersionCatalogLibs(extensions.getByType(VersionCatalogsExtension::class.java).named("libs"))

abstract class CargoPlugin : Plugin<Project> {
  override fun apply(project: Project) {
    with(project) {
      val cargo = extensions.create(CargoExtension.Companion.NAME, CargoExtension::class.java)

      configurePlugins()
      configureDesktopLibs()
      configureDependencies(cargo)
      cargo.configureLibraries()
      cargo.configureBinaries()
      cargo.configureBindgenDirs(layout.buildDirectory.dir("uniffiBindgen"))
      cargo.configureJniLibs(layout.buildDirectory.dir("rustJniLibs"))
      cargo.configureDesktopLibsJar(layout.buildDirectory.dir("rustJniLibs/jars"))
      configureAndroidLibraryComponents(cargo)
    }
  }
}

private fun Project.configureAndroidLibraryComponents(cargo: CargoExtension) {
  extensions.configure(LibraryAndroidComponentsExtension::class.java) {
    mapOf("debug" to JniLibsVariant.AndroidDebug, "release" to JniLibsVariant.AndroidRelease)
        .forEach { (name, variant) ->
          onVariants(selector().withBuildType(name)) {
            it.sources.jniLibs!!.addGeneratedSourceDirectory(
                cargo.jniLibsBridge("copy${name.capitalized()}JniLibs", variant)) {
                  it.outputDirectory
                }
          }
        }

    val out = objects.directoryProperty()
    out.set(layout.buildDirectory.dir("abc"))
    val t =
        tasks.register<Copy>("mergingStuff") {
          from(cargo.bindgenDirs.map { it.values.map { it.map { it.asFile.path } } })
          into(out)
        }
    val b =
        tasks.register<BridgeTask>("bridging") {
          dependsOn(t)
          inputDirectory.set(out)
        }
    onVariants {
      it.sources.java!!.addGeneratedSourceDirectory(b) {
        it.outputDirectory
      }
    }
  }
}

private fun Project.configurePlugins() {
  with(pluginManager) {
    apply(libs.androidLibrary)
    apply(libs.kotlinAndroid)
  }
}

private fun Project.configureDependencies(cargo: CargoExtension) {
  dependencies {
    with(configurations) {
      add(implementation.name, libs.jnaAar)
      add(implementation.name, libs.coroutinesAndroid)
      add(desktopLibs.name, libs.jnaJar)
      add(desktopDebugLibs.name, project.files(cargo.jniLibsDesktopDebug.flatMap { it.map { it.outputs.files.singleFile } }))
      add(desktopReleaseLibs.name, project.files(cargo.jniLibsDesktopRelease.flatMap { it.map { it.outputs.files.singleFile } }))
    }
  }
}

private fun Project.configureDesktopLibs() {
  fun Configuration.configureBase() {
    isCanBeResolved = true
    isCanBeConsumed = true
  }

  val desktopLibs = configurations.register("desktopLibs") { configureBase() }

  val desktopDebugLibs =
      configurations.register("desktopDebugLibs") {
        configureBase()
        extendsFrom(desktopLibs.get())
      }

  val desktopReleaseLibs =
      configurations.register("desktopReleaseLibs") {
        configureBase()
        extendsFrom(desktopLibs.get())
      }

  listOf(
          "testImplementation" to desktopLibs,
          "testDebugImplementation" to desktopDebugLibs,
          "testReleaseImplementation" to desktopReleaseLibs)
      .forEach { (name, extendsFromConfig) ->
        configurations.named(name) { extendsFrom(extendsFromConfig.get()) }
      }
}
