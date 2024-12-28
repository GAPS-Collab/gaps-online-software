# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import sys
import os
sys.path.insert(0, os.path.abspath('/home/achim/mcmurdo/gaps-online-software/build/install/gaps-online-sw-v0.10/python'))

project = 'gaps_online'
copyright = '2024, J.A.Stoessl'
author = 'J.A.Stoessl'
release = '0.10'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
        'sphinx.ext.autodoc',
        'sphinx.ext.autosummary',
        'myst_parser',
        ]

autosummary_generate = True
templates_path = ['_templates']
exclude_patterns = []

language = 'python'

autodoc_default_options = {
    'members': True,
    'member-order': 'bysource',
    'special-members': '__init__',
    'undoc-members': True,
    'exclude-members': '__weakref__'
}

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

#html_theme = 'alabaster'
html_theme = 'pydata_sphinx_theme'
html_theme_options = {'navbar_end': []} 
html_static_path = ['_static']
