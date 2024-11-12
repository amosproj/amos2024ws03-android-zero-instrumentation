// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

import com.nishtahir.CargoBuildTask
import org.gradle.internal.extensions.stdlib.capitalized

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.rust.android)
    alias(libs.plugins.org.cyclonedx.bom)
}

val rustDir = rootProject.file("../rust")
val linuxTarget = "linux-x86-64"
val rustTargets = listOf("arm64", "x86_64", linuxTarget)
val rustLibName = "client" // This has to match the name in the Cargo.toml
fun generatedDir(subdir: String) =
    layout.buildDirectory.file("generated/source/uniffi/${subdir}/java").get().asFile


android {
    namespace = "de.amosproj3.ziofa.client"
    compileSdk = 35
    buildToolsVersion = "35.0.0"
    ndkVersion = "28.0.12433566"

    defaultConfig {
        minSdk = 33

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    sourceSets {
        getByName("debug") {
            kotlin.srcDir(generatedDir("debug"))
        }
        getByName("release") {
            kotlin.srcDir(generatedDir("release"))
        }
    }
}

cargo {
    module = rustDir.path
    libname = rustLibName
    targets = rustTargets
    features {
        defaultAnd(arrayOf("uniffi"))
    }
}

val desktopLibsJar = tasks.register<Jar>("desktopLibsJar") {
    archiveBaseName = "desktop"

    dependsOn(tasks.getByName("cargoBuild"))

    from(layout.buildDirectory.dir("rustJniLibs/desktop").get().asFile)
    destinationDirectory.set(layout.buildDirectory.file("rustJniLibs").get().asFile)
}

dependencies {

    implementation(libs.jna) { artifact { type = "aar" } }
    testImplementation(libs.jna)
    testImplementation(files(desktopLibsJar))

    implementation(libs.androidx.core.ktx)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

tasks.cyclonedxBom {
    setSchemaVersion("1.5")
    setIncludeConfigs(listOf("runtimeClasspath"))
    setOutputName("bom")
    setOutputFormat("json")
    setDestination(project.file("build/reports"))
    setIncludeBomSerialNumber(false)
    setIncludeLicenseText(true)
    setIncludeMetadataResolution(true)
}

afterEvaluate {
    android.libraryVariants.forEach { variant ->

        val task = tasks.register<Exec>("generate${variant.name}UniFFIBindings") {
            val cargoTask =
                tasks.getByName<CargoBuildTask>("cargoBuild${linuxTarget.capitalized()}")
            dependsOn(cargoTask)
            workingDir = rustDir
            commandLine(
                "cargo",
                "run",
                "--bin=uniffi-bindgen",
                "--features=uniffi",
                "--features=uniffi/cli",
                "generate",
                "--language=kotlin",
                "--library",
                layout.buildDirectory.file("rustJniLibs/${cargoTask.toolchain!!.folder}/lib${rustLibName}.so")
                    .get().asFile.path,
                "--out-dir", generatedDir(variant.name)
            )
        }

        tasks.getByName("compile${variant.name.capitalized()}Kotlin").dependsOn(task)
    }

    val cargoBuild = tasks.getByName("cargoBuild")
    android.libraryVariants.forEach { variant ->
        val productFlavor = variant.productFlavors.map { it.name.capitalized() }.joinToString { "" }
        val buildType = variant.buildType.name.capitalized()

        tasks.getByName("generate${productFlavor}${buildType}Assets").dependsOn(cargoBuild)
    }
}

