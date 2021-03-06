import os
import re
import zipfile
import requests
import productdb_pb2
from lxml import html

def prod_version():
    ''' Return current live version of ELVUI '''
    tukui_api = requests.get("https://www.tukui.org/api.php?classic-tbc-addons")
    tukui_api = tukui_api.json()
    for addon in tukui_api:
        if addon["name"] == "ElvUI":
            online_version = addon["version"]
            break
    return online_version

def local_version(wow_dir):
    ''' Return version of local ELVUI install '''
    toc_loc = wow_dir + '\\interface\\addons\\ElvUI\\ElvUI_TBC.toc'
    # Read addon TOC file
    try:
        toc = open(toc_loc, 'r')
        toc_lines = toc.readlines()
        toc.close()
    except FileNotFoundError:
        return 'Not Installed'

    # Parse version string
    version = re.search('(?<=Version: ).*', toc_lines[3])
    return version.group(0)

def update(wow_dir):
    ''' Download and install newest ELVUI '''
    # Download zip
    local_filename = 'elvui.zip'
    req = requests.get('https://www.tukui.org/classic-tbc-addons.php?download=2', stream=True)

    with open(local_filename, 'wb') as download:
        for chunk in req.iter_content(chunk_size=1024):
            if chunk:
                download.write(chunk)

    # Unzip
    elvui_zip = zipfile.ZipFile(local_filename, 'r')

    # Extract to addons folder
    elvui_zip.extractall(path='{:s}{:s}'.format(wow_dir, '\\interface\\addons\\'))

    # Cleanup
    elvui_zip.close()
    os.remove(local_filename)

def main():
    ''' Update if version mismatch '''
    # Identify local install
    wow_dir = installpath()
    # Get local and prod versions
    local = local_version(wow_dir)
    prod = prod_version()

    print('Installed Version: {}'.format(local))
    print('Live Version: {}'.format(prod))

    if local != prod:
        print('Updating...')
        update(wow_dir)
        print('Update Complete')


def installpath():
    productdb_path = os.getenv('ALLUSERSPROFILE') + "\\Battle.net\\Agent\\product.db"
    if os.path.exists(productdb_path):
        f = open(productdb_path, "rb")
        db = productdb_pb2.Database()
        db.ParseFromString(f.read())
        f.close()
        productinstalls = db.productInstall
        for entry in productinstalls:
            if entry.productCode == "wow_classic":
                wow_classic_path = entry.settings.installPath.replace("/", "\\")
                break
        return wow_classic_path
    else:
        return ""

if __name__ == '__main__':
    main()
