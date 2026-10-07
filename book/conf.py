# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'The Bobcat-sdk Book'
copyright = '2026, Bayge'
author = 'Bayge'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = []

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']



# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'alabaster'
html_static_path = ['_static']
html_logo = '../logo.svg'
pygments_style = 'monokai'
html_theme_options = {
    'description': 'A Rust SDK for Arbitrum Stylus',
    'logo_name': True,
    'github_user': 'stylus-developers-guild',
    'github_repo': 'bobcat-sdk',
}
