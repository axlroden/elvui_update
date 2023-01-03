import os
import re
import time
import zipfile
import requests
import productdb_pb2
import urllib.request
from inscriptis import get_text


def query_api(apiurl):
    ''' Return current live version of ELVUI and url '''
    tukui_api = requests.get(apiurl)
    tukui_api = tukui_api.json()
    for addon in tukui_api:
        if addon["name"] == "ElvUI":
            online_version = addon["version"]
            url = addon["url"]
            break
    return online_version, url

def local_version(folder):
    ''' Return version of local ELVUI install '''
    toc_loc = folder + 'ElvUI\\ElvUI_Mainline.toc'
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


def update(folder, url):
    ''' Download and install newest ELVUI '''
    # Download zip
    local_filename = 'elvui.zip'
    req = requests.get(url, stream=True)

    with open(local_filename, 'wb') as download:
        for chunk in req.iter_content(chunk_size=1024):
            if chunk:
                download.write(chunk)

    # Unzip
    elvui_zip = zipfile.ZipFile(local_filename, 'r')

# Extract to addons folder
    elvui_zip.extractall(path='{:s}'.format(folder))
    # Cleanup
    elvui_zip.close()
    os.remove(local_filename)

def main():
    ''' Update if version mismatch '''
    # Identify local install
    classic_path, retail_path = installpath()
    updated = False
    # Get local and prod versions
    # The live API does not have ElvUI ...
    prod, url = query_api('https://www.tukui.org/api.php?classic-wotlk-addons')
    if classic_path is not None:
        classic_local = local_version(classic_path)
        print('Installed classic version: {}'.format(classic_local))
        print('Current classic version: {}'.format(prod))
        # print('Url: {}'.format(url))
        if classic_local != prod:
            print('Updating..')
            update(classic_path, url)
            print('Update complete')
            updated = True

    if retail_path is not None:
        retail_local = local_version(retail_path)
        print('Installed retail version: {}'.format(retail_local))
        print('Current retail version: {}'.format(prod))
        # print('Url: {}'.format(url))
        if retail_local != prod:
            print('Updating..')
            update(retail_path, url)
            print('Update complete')
            updated = True
    if updated is True:
        url = "https://www.tukui.org/ui/elvui/changelog"
        html = urllib.request.urlopen(url).read().decode('utf-8')
        current_changelog = html.split("<u>", 2)
        text = get_text(current_changelog[1])
        print(text)
        # sleep 60 secs so there is time to read.
        time.sleep(60)

def installpath():
    productdb_path = os.getenv('ALLUSERSPROFILE') + "\\Battle.net\\Agent\\product.db"
    wow_classic_path = None
    wow_retail_path = None
    if os.path.exists(productdb_path):
        f = open(productdb_path, "rb")
        db = productdb_pb2.Database()
        db.ParseFromString(f.read())
        f.close()
        productinstalls = db.productInstall
        for entry in productinstalls:
            if entry.productCode == "wow_classic":
                wow_classic_path = entry.settings.installPath.replace("/", "\\")
                wow_classic_path = wow_classic_path + "\\_classic_\\interface\\addons\\"
            if entry.productCode == "wow":
                wow_retail_path = entry.settings.installPath.replace("/", "\\")
                wow_retail_path = wow_retail_path + "\\_retail_\\interface\\addons\\"
    return wow_classic_path, wow_retail_path

if __name__ == '__main__':
    main()
