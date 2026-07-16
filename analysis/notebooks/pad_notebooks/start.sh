export PYTHONNOUSERSITE=1
cd /home/gaps/software/old2/gaps-online-software
git pull origin AULEPE-0.12
/home/gaps/software/old2/gaps_os_pro/analysis/notebooks/.venv/bin/python -m pip install -U /home/gaps/software/old2/gaps-online-software/gondola-core/rust/gondola-core
source ../.venv/bin/activate
