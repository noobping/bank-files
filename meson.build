project(
  'bank-files',
  version: '0.1.0',
  meson_version: '>=1.1.0',
)

gnome = import('gnome')
i18n = import('i18n')
cargo = find_program('cargo', required: false)
flatpak_spawn = find_program('flatpak-spawn', required: false)
shell = find_program('sh')

cargo_variant = get_option('cargo_variant')
cargo_features = get_option('cargo_features')
setup = get_option('setup')
project_binary = 'bank-files'
translation_domain = project_binary

datadir_option = get_option('datadir')
if datadir_option.startswith('/')
  installed_datadir = datadir_option
else
  installed_datadir = join_paths(get_option('prefix'), datadir_option)
endif

effective_cargo_features = cargo_features
foreach feature_name, enabled : {
  'setup': setup,
}
  if enabled
    if effective_cargo_features == ''
      effective_cargo_features = feature_name
    else
      effective_cargo_features += ' ' + feature_name
    endif
  endif
endforeach

native_cargo = ''
flatpak_spawn_path = ''
if cargo.found()
  native_cargo = cargo.full_path()
  message('Configured Meson wrapper cargo source "native cargo" -> @0@'.format(native_cargo))
elif flatpak_spawn.found()
  flatpak_spawn_path = flatpak_spawn.full_path()
  message('Configured Meson wrapper cargo source "flatpak-spawn --host toolbox run cargo"')
else
  error('Neither cargo nor flatpak-spawn was found. Install cargo in the build environment or provide flatpak-spawn access to host toolbox.')
endif

if flatpak_spawn.found()
  flatpak_spawn_path = flatpak_spawn.full_path()
endif

message('Configured Meson wrapper variant "@0@"'.format(cargo_variant))
if effective_cargo_features != ''
  message('Configured Meson wrapper features "@0@"'.format(effective_cargo_features))
endif

feature_tokens = []
foreach feature_token : effective_cargo_features.replace(',', ' ').split(' ')
  if feature_token != ''
    feature_tokens += [feature_token]
  endif
endforeach

local_ai_feature_enabled = feature_tokens.contains('local-ai') or feature_tokens.contains('embedded-ai-model') or feature_tokens.contains('setup')
embedded_ai_feature_enabled = feature_tokens.contains('embedded-ai-model') or feature_tokens.contains('setup')
install_local_ai_assets = local_ai_feature_enabled and not embedded_ai_feature_enabled

message('Configured Meson install data dir "@0@"'.format(installed_datadir))

cargo_wrapper_script = '''
set -eu

native_cargo=$1
flatpak_spawn=$2
source_root=$3
build_root=$4
variant=$5
features=$6
install_datadir=$7
output=$8
output_path=$output

profile_dir=debug

if [ "$variant" = "release" ] || [ "$variant" = "translation" ]; then
  profile_dir=release
fi

case "$output_path" in
  /*) ;;
  *) output_path="$build_root/$output_path" ;;
esac

cargo_target_dir="$build_root/cargo-target"
export BANK_FILES_DATADIR="$install_datadir"

if [ -n "$native_cargo" ]; then
  set -- env "BANK_FILES_DATADIR=$install_datadir" "$native_cargo" build "--manifest-path" "$source_root/Cargo.toml" "--target-dir" "$cargo_target_dir" "--locked"
else
  cargo_target_dir="$source_root/target/meson-toolbox"
  cd "$source_root"
  set -- "$flatpak_spawn" --host toolbox run env "BANK_FILES_DATADIR=$install_datadir" cargo build "--manifest-path" "$source_root/Cargo.toml" "--target-dir" "$cargo_target_dir" "--locked"
fi

if [ "$profile_dir" = "release" ]; then
  set -- "$@" --release
fi

if [ -n "$features" ]; then
  set -- "$@" --features "$features"
fi

"$@"

mkdir -p "$(dirname "$output_path")"
cp "$cargo_target_dir/$profile_dir/bank-files" "$output_path"
chmod +x "$output_path"
'''

cargo_wrapper_args = [
  shell,
  '-c',
  cargo_wrapper_script,
  'meson-cargo-toolbox',
  native_cargo,
  flatpak_spawn_path,
  meson.project_source_root(),
  meson.project_build_root(),
  installed_datadir,
]

custom_target(
  'bank-files-default-build',
  output: project_binary,
  command: cargo_wrapper_args + [
    cargo_variant,
    effective_cargo_features,
    '@OUTPUT@',
  ],
  build_always_stale: true,
  build_by_default: true,
  console: true,
  install: true,
  install_dir: get_option('bindir'),
)

foreach variant : ['developer', 'release', 'translation']
  run_target(
    variant + '-build',
    command: cargo_wrapper_args + [
      variant,
      effective_cargo_features,
      join_paths(meson.project_build_root(), 'artifacts', variant, project_binary),
    ],
  )
endforeach

install_data(
  'data/scalable/apps/io.github.noobping.BankFiles.svg',
  install_dir: join_paths(get_option('datadir'), 'icons', 'hicolor', 'scalable', 'apps'),
)

install_data(
  'data/symbolic/apps/io.github.noobping.BankFiles-symbolic.svg',
  install_dir: join_paths(get_option('datadir'), 'icons', 'hicolor', 'symbolic', 'apps'),
)

install_subdir(
  'data/symbolic/actions',
  install_dir: join_paths(get_option('datadir'), 'icons', 'hicolor', 'symbolic'),
)

install_data(
  'data/io.github.noobping.BankFiles.desktop',
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

install_data(
  'data/bank-files-search-provider.ini',
  rename: 'io.github.noobping.BankFiles.search-provider.ini',
  install_dir: join_paths(get_option('datadir'), 'gnome-shell', 'search-providers'),
)

install_data(
  'data/bank-files-search-provider.service',
  rename: 'io.github.noobping.BankFiles.SearchProvider.service',
  install_dir: join_paths(get_option('datadir'), 'dbus-1', 'services'),
)

install_data(
  'data/metainfo.xml',
  rename: 'io.github.noobping.BankFiles.metainfo.xml',
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

install_data(
  'data/io.github.noobping.BankFiles.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0', 'schemas'),
)

if install_local_ai_assets
  install_subdir(
    'data/ai',
    install_dir: join_paths(get_option('datadir'), project_binary, 'models'),
  )
endif

translation_languages = ['nl', 'de']
subdir('po')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
