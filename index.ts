import axios from "axios";
import { createWriteStream, readFileSync, writeFileSync } from "fs";
import { mkdirSync, existsSync } from "fs-extra";
import { writeFile } from "fs/promises";
import path from "path";

type mojangVersionManifest = {
  versions: {
    id: string;
    type: string;
    url: string;
    time: string;
    releaseTime: string;
    sha1: string;
    complianceLevel: number;
  }[];
};

type fabricManifest = {
  mainClass: string;
  arguments: {
    game: string[];
    jvm: string[];
  };
  libraries: { name: string; url: string; sha1: string }[];
};

function resolveMaven(file: string) {
  const [path, name, version, ...more] = file.split(":");
  return `${path.replace(/\./g, "/")}/${name}/${version}/${name}-${version}${
    more.length ? `-${more.join("-")}` : ""
  }.jar`;
}

export type osType = "windows" | "linux" | "osx";

type mojangClientManifest = {
  assetIndex: {
    id: string;
  };
  libraries: [
    {
      downloads: {
        artifact: {
          path: string;
          sha1: string;
          size: number;
          url: string;
        };
        classifiers: {
          [key: string]: {
            path: string;
            sha1: string;
            size: number;
            url: string;
          };
        };
      };
      name: string;
      rules: {
        action: string;
        os: {
          name: osType;
        };
      }[];
    }
  ];
  downloads: {
    client: {
      sha1: string;
      size: number;
      url: string;
    };
    client_mappings: {
      sha1: string;
      size: number;
      url: string;
    };
  };
};

async function createForgeManifest(
  server_name: string,
  forge_versrion: string,
  mc_version: string
) {
  const version_manifest = (
    await axios.get(
      "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
    )
  ).data as mojangVersionManifest;

  const mc_version_manifest = version_manifest.versions.find(
    (v) => v.id === mc_version
  )?.url;

  if (!mc_version_manifest) {
    throw new Error("MC version not found");
  }

  const mojang_manifest = (await axios.get(mc_version_manifest))
    .data as mojangClientManifest;

  const libraries = new Set<string>();

  const file = readFileSync(forge_versrion + ".json");

  const json: {
    mainClass: string;
    libraries: {
      name: string;
      downloads: {
        artifact: {
          path: string;
          sha1: string;
          size: number;
          url: string;
        };
      };
    }[];
    minecraftArguments?: string;
    arguments: {
      game: string[];
      jvm: string[];
    };
  } = JSON.parse(file.toString());

  json.libraries.forEach((l) => {
    if (!l.downloads.artifact?.path) return;

    const libraryObj = {
      path: "libraries/" + l.downloads.artifact.path,
      os: ["windows", "linux", "osx"] as osType[],
    };

    libraries.add(JSON.stringify(libraryObj));
  });

  console.log(mojang_manifest.libraries);

  mojang_manifest.libraries?.forEach((l) => {
    if (!l.downloads.artifact?.path) return;

    const oss = new Set<string>(["windows", "linux", "osx"]);

    l.rules?.forEach((rule) => {
      if (rule.action === "disallow") oss.delete(rule.os.name);
      if (rule.action === "allow" && rule?.os?.name) {
        oss.clear();
        oss.add(rule.os.name);
      }
    });

    const includesWindows = l.downloads.artifact.path.includes("windows");
    const includesLinux = l.downloads.artifact.path.includes("linux");
    const includesOSX = l.downloads.artifact.path.includes("macos");

    if (includesWindows) {
      oss.delete("linux");
      oss.delete("osx");
    }
    if (includesLinux) {
      oss.delete("windows");
      oss.delete("osx");
    }
    if (includesOSX) {
      oss.delete("windows");
      oss.delete("linux");
    }

    const libraryObj = {
      path: "libraries/" + l.downloads.artifact.path,
      os: Array.from(oss) as osType[],
    };

    libraries.add(JSON.stringify(libraryObj));
  });

  // После добавления всех элементов, преобразуем JSON обратно в объекты
  const uniqueLibraries = Array.from(libraries).map((lib) => JSON.parse(lib));

  if (!existsSync(server_name)) mkdirSync(server_name);

  const all_libs = [
    ...mojang_manifest.libraries,
    ...json.libraries,
    {
      downloads: {
        artifact: {
          path: "clients/client" + mc_version + ".jar",
          url: mojang_manifest.downloads.client.url,
        },
      },
    },
    {
      downloads: {
        artifact: {
          path: "clients/client" + mc_version + ".txt",
          url: mojang_manifest.downloads.client.url,
        },
      },
    },
  ];

  all_libs.forEach(async (l) => {
    if (!l.downloads.artifact?.path) return;

    const filePath = path.join(
      server_name,
      "libraries",
      l.downloads.artifact.path
    );

    // Отделяем путь к директории от полного пути файла
    const dirPath = path.dirname(filePath);

    // Создаем директорию, если она не существует
    if (!existsSync(dirPath)) {
      mkdirSync(dirPath, { recursive: true });
    }

    // Остальная часть кода остается неизменной
    const writer = createWriteStream(filePath);

    console.log("Downloading " + l.downloads.artifact.url);

    const response = await axios({
      url: l.downloads.artifact.url,
      method: "GET",
      responseType: "stream",
    })
      .catch((e) => console.log(e))
      .then((data) => data);

    if (response) {
      response.data.pipe(writer);
    } else {
      console.log(l.downloads.artifact.url);
    }

    return new Promise((resolve, reject) => {
      writer.on("finish", () => {
        console.log("Downloaded " + l.downloads.artifact.path);
        resolve();
      });

      writer.on("error", reject);
    });
  });

  const jvm = [
    "-XX:+IgnoreUnrecognizedVMOptions",
    "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump",
    "-XX:+DisableAttachMechanism",
    "-XX:+UnlockExperimentalVMOptions",
    "-Xss1M",
    "-XX:+UseG1GC",
    "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20",
    "-XX:MaxGCPauseMillis=50",
    "-XX:G1HeapRegionSize=32M",
  ];

  if (json?.arguments?.jvm) jvm.push(...json.arguments.jvm);

  writeFileSync(
    server_name + "/manifest.json",
    JSON.stringify({
      mainclass: json.mainClass,
      assetIndex: mojang_manifest.assetIndex.id,
      libraries: [
        ...uniqueLibraries,
        {
          path: "minecraft.jar",
          os: ["windows", "linux", "osx"],
        },
      ],
      jvm: jvm,
      game: json.minecraftArguments
        ? json.minecraftArguments.split(" ").slice(-4)
        : json.arguments.game,
      verify: ["mods", "libraries", "minecraft.jar"],
      ignore: ["options.txt"],
    })
  );
}

async function createFabricManifest(
  server_name: string,
  fabric_version: string,
  mc_version: string
) {
  const version_manifest = (
    await axios.get(
      "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
    )
  ).data as mojangVersionManifest;

  const mc_version_manifest = version_manifest.versions.find(
    (v) => v.id === mc_version
  )?.url;

  if (!mc_version_manifest) {
    throw new Error("MC version not found");
  }

  const mojang_manifest = (await axios.get(mc_version_manifest))
    .data as mojangClientManifest;

  const fabric_manifest: fabricManifest = (
    await axios.get(
      `https://meta.fabricmc.net/v2/versions/loader/${mc_version}/${fabric_version}/profile/json`
    )
  ).data;

  const libraries = new Set<string>();

  fabric_manifest.libraries.forEach((l) => {
    const libraryObj = {
      path: "libraries/" + resolveMaven(l.name),
      os: ["windows", "linux", "osx"] as osType[],
    };

    libraries.add(JSON.stringify(libraryObj));
  });

  mojang_manifest.libraries.forEach((l) => {
    if (!l.downloads.artifact?.path) return;

    const oss = new Set<string>(["windows", "linux", "osx"]);

    l.rules?.forEach((rule) => {
      if (rule.action === "disallow") oss.delete(rule.os.name);
    });

    const includesWindows = l.downloads.artifact.path.includes("windows");
    const includesLinux = l.downloads.artifact.path.includes("linux");
    const includesOSX = l.downloads.artifact.path.includes("macos");

    if (includesWindows) {
      oss.delete("linux");
      oss.delete("osx");
    }
    if (includesLinux) {
      oss.delete("windows");
      oss.delete("osx");
    }
    if (includesOSX) {
      oss.delete("windows");
      oss.delete("linux");
    }

    const libraryObj = {
      path: "libraries/" + l.downloads.artifact.path,
      os: Array.from(oss) as osType[],
    };

    libraries.add(JSON.stringify(libraryObj));
  });

  // После добавления всех элементов, преобразуем JSON обратно в объекты
  const uniqueLibraries = Array.from(libraries).map((lib) => JSON.parse(lib));

  if (!existsSync(server_name)) mkdirSync(server_name);

  const all_libs = [
    ...mojang_manifest.libraries.map((l) => ({
      name: l.name,
      url: l.downloads.artifact.url,
      sha1: l.downloads.artifact.sha1,
    })),
    ...fabric_manifest.libraries.map((l) => ({
      name: l.name,
      url: l.url + resolveMaven(l.name),
    })),
  ];

  all_libs.forEach(async (l) => {
    if (!l.name) return;

    const normalPath = resolveMaven(l.name);

    const filePath = path.join(server_name, "libraries", normalPath);

    // Отделяем путь к директории от полного пути файла
    const dirPath = path.dirname(filePath);

    // Создаем директорию, если она не существует
    if (!existsSync(dirPath)) {
      mkdirSync(dirPath, { recursive: true });
    }

    const response = await axios({
      url: l.url,
      method: "GET",
      responseType: "stream",
    });

    const writeStream = createWriteStream(filePath);

    response.data.pipe(writeStream);

    await new Promise((resolve, reject) => {
      writeStream.on("finish", resolve);
      writeStream.on("error", reject);
    });

    console.log("Downloaded " + normalPath);
  });

  const jvm = [
    "-XX:+IgnoreUnrecognizedVMOptions",
    "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump",
    "-XX:+DisableAttachMechanism",
    "-XX:+UnlockExperimentalVMOptions",
    "-Xss1M",
    "-XX:+UseG1GC",
    "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20",
    "-XX:MaxGCPauseMillis=50",
    "-XX:G1HeapRegionSize=32M",
  ];

  if (fabric_manifest?.arguments?.jvm)
    jvm.push(...fabric_manifest.arguments.jvm);

  writeFileSync(
    server_name + "/manifest.json",
    JSON.stringify({
      mainclass: fabric_manifest.mainClass,
      assetIndex: mojang_manifest.assetIndex.id,
      libraries: [
        ...uniqueLibraries,
        {
          path: "minecraft.jar",
          os: ["windows", "linux", "osx"],
        },
      ],
      jvm: jvm,
      game: fabric_manifest.arguments.game,
      verify: ["mods", "libraries", "minecraft.jar"],
      ignore: ["options.txt"],
    })
  );
}

createForgeManifest("saom", "1.16.5-forge-36.2.41", "1.16.5");
// createFabricManifest("shizik", "0.15.4", "1.20.1");
