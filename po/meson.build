i18n.gettext(
  translation_domain,
  languages: translation_languages,
  install: true,
)

install_data(
  'nl.po',
  'de.po',
  'LINGUAS',
  'POTFILES',
  install_dir: join_paths(get_option('datadir'), project_binary, 'po'),
)

msgattrib = find_program('msgattrib', required: false)
if msgattrib.found()
  custom_target(
    translation_domain + '-template',
    input: 'nl.po',
    output: translation_domain + '.pot',
    command: [msgattrib, '--no-obsolete', '--empty', '@INPUT@', '-o', '@OUTPUT@'],
    install: true,
    install_dir: join_paths(get_option('datadir'), project_binary, 'po'),
  )
else
  warning('msgattrib was not found; the translation template will not be installed.')
endif
