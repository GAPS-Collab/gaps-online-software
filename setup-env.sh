#! /bin/sh
export PATH=$PATH:/srv/gaps/gaps-online-software/install/gaps-online-sw-v0.8/bin
export PYTHONPATH=$PYTHONPATH:/srv/gaps/gaps-online-software/install/gaps-online-sw-v0.8/python:/srv/gaps/gaps-online-software/gaps-db/gaps_db
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/srv/gaps/gaps-online-software/install/gaps-online-sw-v0.8/lib
export DJANGO_SETTINGS_MODULE=gaps_db.settings


echo -e "\n\n  **************************************************"
echo -e "  * WELCOME TO GAPS-ONLINE-SOFTWARE V0.8 'NIUHI'   *"
echo -e "  **************************************************"
echo -e "   -- We have set the following variables:"
echo -e "   -- PYTHONPATH=$PYTHONPATH"
echo -e "   -- PATH=${PATH//:/\\n  }"
echo -e "   -- LD_LIBRARY_PATH=${LD_LIBRARY_PATH//:/\\n  }"
echo -e "  ********************************************"
echo -e "   => software repository:"
echo -e "      - https://github.com/GAPS-Collab/gaps-online-software"
echo -e "   => Where to get help?:"
echo -e "      - README.md"
echo -e "   => Maintainer?:"
echo -e "      - stoessl@hawaii.edu"

