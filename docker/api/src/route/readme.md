# Général
Ce cas concerne toutes les requêtes qui demande d'être authentifié au préalable

## Output erreur
```
APIError::NoFile=>"600",
            APIError::Metadata=>"601",

            // Cas Généraux
            APIError::NoCookieBearer=>"101",
            APIError::NoAuthAppData=>"102",
            APIError::Script=>"103",
            APIError::Ssh=>"104",
            APIError::Sftp=>"105",
            APIError::ValidInput=>"106",

            // File
            APIError::Write=>"200",

            //Convertion
            APIError::UTF8=>"300",
            APIError::Json=>"301",
            APIError::Usize=>"302",

            //Logup
            APIError::AlreadyExist=>"1",
            APIError::UsernameTooShort=>"2",
            APIError::InvalidPassword=>"3",
            APIError::PasswordTooShort=>"4",
            APIError::SpecialCharMissing=>"5",
            APIError::MajusculeMissing=>"6",
            APIError::NumberMissing=>"7",

            // Login
            APIError::NotSignup=>"0",

            //Bearer
            APIError::Expired=>"Effacement du cookie"
            APIError::ErrorBearer=>"504",

            // token
            APIError::EncodeToken=>"700",
            //Encryption
            APIError::KDFError =>"400"
```

# /api/signup
Lors de l'inscription d'un nouveau utilisateur, celui-ci lui envoie son username et password, il vérifie si l'utilisateur n'est pas déjà enregistré, l'ajoute à la base de données et lui renvoie un cookie d'authentification'.
## input
Type: ```application/json``` | method: ```post```
```
{
    "username": "marc-antoine.dumar@gmail.com",
    "password": "Tetris123@"
}
```
## output
```status code:``` 200
```
Set-Cookie Bearer=<JWT_Token>
```


# /api/signin
Lors de la connection d'un utilisateur, celui-ci lui envoie son username et password, vérifie s'il est déjà enregistrer et lui renvoie son cookie d'authentification
## input
Type: ```application/json``` | method: ```post```
```
{
    "username": "marc-antoine.dumar@gmail.com",
    "password": "Tetris123@"
}
```
## output
```status code:``` 200
```
Set-Cookie Bearer=<JWT_Token>
```


# /api/get_repot_key
Une fois l'utilisateur authentifier avec son cookie, on lui envoie sous forme de fichier téléchargeable sa clé master 1.
## input
```
Cookie Bearer=<JWT_Token>
```
## Output
```status code:``` 200
Type: ```application/octet-stream```
```
<repot_key_encrypted>
```


# /api/get_ssh_pub_key_server
Envoie à l'utilisateur de la clé ssh publique du serveur pour qu'il se connecte à l'utilisateur
## Input
Type: ```application/json``` | method: ```post```

## Output
```status code:``` 200
Type: ```application/json```
```
{
    ssh_pub: <ssh_key_value>
}
```

# /api/send_ssh_key_tunnel
L'utilisateur envoie au serveur la clé ssh publique pour se connecter à l'utilisateur tunnel sur le serveur
## Input
```status code:``` 200
Type: ```application/json``` | method: ```post```
```
{
    ssh_key: <ssh_key_value>
}
```
## Output
```status code:``` 200

# /api/send_ssh_key
Une fois l'utilisateur authentifier avec son cookie, il nous envoie sa clé ssh publique sous forme d'un fichier,on lui renvoie un status OK.
## Input
```
Cookie Bearer=<JWT_Token>
```
Type: ```application/json```
```
{
    ssh_key: <ssh_key_value>
}
```
## Output
Status code: ```200```

# /api/get_list
Une fois l'utilisateur authentifier avec son cookie, il demande le contenue de repot Borg sous forme d'un json.
## input
```
Cookie Bearer=<JWT_Token>
```
body vide pour lister les archives disponibles ou
Type: ```application/json```
```
{
    archive_name: <archive_name>
}
```
pour lister le contenu des archive
## output
Status code: ```200```

Type: ```application/json```
```
{
    "archives": [
        {
            "archive": "2026-02-18_11-43-46",
            "time": "2026-02-18T10:43:50.000000"
        },
        {
            "archive": "2026-02-18_11-43-46_logs",
            "time": "2026-02-18T10:44:01.000000"
        },
    ]
}
```
ou avec archive_name spécifier
```
{
    "archive_name": "<nom_de_l'archive>"
    "archive_content": [
        {
            "type": "-", 
            "path": "mnt/c/Users/arthu/Documents/Analyse Fonctionnelle/BeteACorne.png", 
            "mtime": "2025-11-07T13:51:02.354852", "size": 108736
        },
        {
            "type": "-", 
            "path": "mnt/c/Users/arthu/Documents/Analyse Fonctionnelle/BeteACorne.png", 
            "mtime": "2025-11-07T13:51:02.354852", "size": 108736
        }
    ]
}
```
# /api/get_restore
## input
```
Cookie Bearer=<JWT_Token>
```
```
{
    archive_name: <archive_name>
}
```
ou pour retourner que un fichier
```
{
    "archive_name": "2026-02-18_16-36-55",
    "file_name": "mnt/d/ACBF Remake/Audio2.m4a"
}
```
## output
Type: ```application/octet-stream```
```
<archive>.tar.gz
```
ou
```
<file_name>
```

# api/get_log
## input
```
Cookie Bearer=<JWT_Token>
```
## output
```
{
    "logs": [
        {
            "files": [
                "A /mnt/d/2025-12-17 14-47-33.mkv"
            ],
            "log": {
                "archive": {
                    "command_line": [
                        "/usr/bin/borg",
                        "create",
                        "--compression",
                        "zstd,6",
                        "--stats",
                        "--list",
                        "--json",
                        "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo::2026-02-18_11-43-46",
                        "--patterns-from",
                        "/mnt/c/Users/arthu/AppData/Roaming/com.strongholder.client/patterns_tespr.lst"
                    ],
                    "duration": 7.036595,
                    "end": "2026-02-18T11:43:57.000000",
                    "id": "3cd77bc82fd34c7ed792fe1486791518c04501138315780548fbdc3f843d10d3",
                    "limits": {
                        "max_archive_size": 0.00003199583586832383
                    },
                    "name": "2026-02-18_11-43-46",
                    "stats": {
                        "compressed_size": 9251345,
                        "deduplicated_size": 9251345,
                        "nfiles": 1,
                        "original_size": 9806960
                    }
                },
                "cache": {
                    "path": "/home/hugo/.cache/borg/f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "stats": {
                        "total_chunks": 8,
                        "total_csize": 9250674,
                        "total_size": 9806209,
                        "total_unique_chunks": 8,
                        "unique_csize": 9251807,
                        "unique_size": 9807384
                    }
                },
                "encryption": {
                    "keyfile": "/home/hugo/.config/borg/keys/71aea833849e4c258f17c381669b1c7c.key",
                    "mode": "keyfile"
                },
                "repository": {
                    "id": "f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "last_modified": "2026-02-18T11:43:59.000000",
                    "location": "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo"
                }
            }
        },
        {
            "files": [
                "U /mnt/d/2025-12-17 14-47-33.mkv"
            ],
            "log": {
                "archive": {
                    "command_line": [
                        "/usr/bin/borg",
                        "create",
                        "--compression",
                        "zstd,6",
                        "--stats",
                        "--list",
                        "--json",
                        "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo::2026-02-18_16-16-54",
                        "--patterns-from",
                        "/mnt/c/Users/arthu/AppData/Roaming/com.strongholder.client/patterns_tespr.lst"
                    ],
                    "duration": 0.031128,
                    "end": "2026-02-18T16:16:59.000000",
                    "id": "26a29984fd3d0d445b8983e107eff06e17889451e1e528d992edbc1f6c62d789",
                    "limits": {
                        "max_archive_size": 0.000032043519677367536
                    },
                    "name": "2026-02-18_16-16-54",
                    "stats": {
                        "compressed_size": 9251346,
                        "deduplicated_size": 672,
                        "nfiles": 1,
                        "original_size": 9806960
                    }
                },
                "cache": {
                    "path": "/home/hugo/.cache/borg/f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "stats": {
                        "total_chunks": 19,
                        "total_csize": 18502081,
                        "total_size": 19614253,
                        "total_unique_chunks": 12,
                        "unique_csize": 9254122,
                        "unique_size": 9810938
                    }
                },
                "encryption": {
                    "keyfile": "/home/hugo/.config/borg/keys/71aea833849e4c258f17c381669b1c7c.key",
                    "mode": "keyfile"
                },
                "repository": {
                    "id": "f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "last_modified": "2026-02-18T16:16:59.000000",
                    "location": "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo"
                }
            }
        },
        {
            "files": [
                "A /mnt/d/ACBF Remake Deception/Audio1.m4a",
                "A /mnt/d/ACBF Remake Deception/Audio2.m4a",
                "A /mnt/d/ACBF Remake Deception/Audio1.wav",
                "A /mnt/d/ACBF Remake Deception/Audio1.aup3",
                "A /mnt/d/ACBF Remake Deception/Audio2.wav",
                "A /mnt/d/ACBF Remake Deception/AudioEditedAI.wav",
                "x /mnt/d/ACBF Remake Deception/AudioEdited.fcpxml",
                "A /mnt/d/ACBF Remake Deception/Audio2.aup3",
                "x /mnt/d/ACBF Remake Deception/Clips",
                "x /mnt/d/ACBF Remake Deception/TestChapter1.mov",
                "x /mnt/d/ACBF Remake Deception/TestChapter1SE.mp4",
                "x /mnt/d/ACBF Remake Deception/Chapter1Test2.mov",
                "x /mnt/d/ACBF Remake Deception/Chapter1Test2SE.mp4",
                "x /mnt/d/ACBF Remake Deception/TestVideo.mov",
                "x /mnt/d/ACBF Remake Deception/TestVideoSE.mp4",
                "x /mnt/d/ACBF Remake Deception/ACBFRemakeDeception.mov",
                "x /mnt/d/ACBF Remake Deception/AC BF Minia",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaBase.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/Jungle.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/Fight.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/Story.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/LootRPG.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/AC4BF_White-Red_Logo_1380793612.jpg",
                "A /mnt/d/ACBF Remake Deception/Shorts/AC15_LOGO_ACBLACKFLAG_20220614_6PM_CEST.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaCombat.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaRPG.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaRPG.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaCombat.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/AC4BF_MidRez_SP_01_Edward_1380794442.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/AC4BF_MidRez_SP_01_EdwardShadow_1380794442.jpg",
                "A /mnt/d/ACBF Remake Deception/Shorts/AC4BF_SC_SP_01_IIconicPose_Edward_1380793851.JPG.jpg",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaRPGOriginal.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaRPGOriginal.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/assassinscreedivblackflag_environment_abstergo_entertainment_working_area_01_by_eddie_bennun_additions_02.jpg",
                "A /mnt/d/ACBF Remake Deception/Shorts/recycle_bin_PNG50.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/pngimg.com - red_arrow_PNG16.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaPRESENT.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaPRESENT.png",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaStory.psd",
                "A /mnt/d/ACBF Remake Deception/Shorts/MiniaStory.png",
                "d /mnt/d/ACBF Remake Deception/Shorts",
                "d /mnt/d/ACBF Remake Deception"
            ],
            "log": {
                "archive": {
                    "command_line": [
                        "/usr/bin/borg",
                        "create",
                        "--compression",
                        "zstd,6",
                        "--stats",
                        "--list",
                        "--json",
                        "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo::2026-02-18_16-36-55",
                        "--patterns-from",
                        "/mnt/c/Users/arthu/AppData/Roaming/com.strongholder.client/patterns_New_Preset.lst"
                    ],
                    "duration": 221.589753,
                    "end": "2026-02-18T16:40:41.000000",
                    "id": "8f2e9ef4eb7bcb2aa3eb24ef8ded8e18bccc66e6e4f2b4938320c58e8407995f",
                    "limits": {
                        "max_archive_size": 0.00003252035776780455
                    },
                    "name": "2026-02-18_16-36-55",
                    "stats": {
                        "compressed_size": 657638076,
                        "deduplicated_size": 632173435,
                        "nfiles": 30,
                        "original_size": 806415335
                    }
                },
                "cache": {
                    "path": "/home/hugo/.cache/borg/f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "stats": {
                        "total_chunks": 387,
                        "total_csize": 676140215,
                        "total_size": 826030668,
                        "total_unique_chunks": 360,
                        "unique_csize": 641444819,
                        "unique_size": 787798378
                    }
                },
                "encryption": {
                    "keyfile": "/home/hugo/.config/borg/keys/71aea833849e4c258f17c381669b1c7c.key",
                    "mode": "keyfile"
                },
                "repository": {
                    "id": "f15bbd7c155bbfdb57c56c6dd2bdf95ed3963d4d5549443eb0712ec3add4d3d7",
                    "last_modified": "2026-02-18T16:40:42.000000",
                    "location": "ssh://71aea833849e4c258f17c381669b1c7c@strongholder.fr:22/srv/repos/71aea833849e4c258f17c381669b1c7c/repo"
                }
            }
        }
    ]
}
```