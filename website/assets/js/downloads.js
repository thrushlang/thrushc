(function () {
  var config = {
    version: "0.2.1",
    base: "https://github.com/thrustlang/thrustc/releases/download/",
    platforms: {
      windows: {
        tag: "thrustc-x86_64-windows-msvc-v{version}",
        file: "thrustc.exe",
        altFile: "thrustc-stripped.exe"
      },
      linux: {
        tag: "thrustc-x86_64-linux-ubuntu-v{version}",
        file: "thrustc",
        altFile: "thrustc-stripped"
      },
      macos_intel: {
        tag: "thrustc-x86_64-macos-v{version}",
        file: "thrustc",
        altFile: "thrustc-stripped"
      },
      macos_arm: {
        tag: "thrustc-aarch64-macos-v{version}",
        file: "thrustc",
        altFile: "thrustc-stripped"
      }
    }
  };

  function fill(template) {
    return template.replace("{version}", config.version);
  }

  function releaseUrl(item, filename) {
    var tag = fill(item.tag);
    return config.base + tag + "/" + filename;
  }

  Object.keys(config.platforms).forEach(function (key) {
    var item = config.platforms[key];
    var primary = document.querySelector('[data-download="' + key + '"]');
    var secondary = document.querySelector('[data-download-alt="' + key + '"]');
    var versionNode = document.querySelector('[data-version="' + key + '"]');

    if (primary) {
      primary.href = releaseUrl(item, item.file);
    }

    if (secondary) {
      secondary.href = releaseUrl(item, item.altFile);
    }

    if (versionNode) {
      versionNode.textContent = "v" + config.version;
    }
  });
})();
