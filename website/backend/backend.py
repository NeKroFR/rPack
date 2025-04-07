from flask import Flask, request, send_file, abort
from flask_cors import CORS
import os
import subprocess
import uuid
from werkzeug.utils import secure_filename

app = Flask(__name__)
CORS(app)  # Activation de CORS pour toutes les routes

# Dossier temporaire pour stocker les fichiers uploadés et générés
UPLOAD_FOLDER = "./tmp"
if not os.path.exists(UPLOAD_FOLDER):
    os.makedirs(UPLOAD_FOLDER)
    print("Création du dossier temporaire:", UPLOAD_FOLDER)
else:
    print("Dossier temporaire existant:", UPLOAD_FOLDER)

@app.route('/process', methods=['POST'])
def process():
    print("=== Requête reçue sur /process ===")

    # Vérifier la présence du fichier dans la requête
    if 'file' not in request.files:
        print("Erreur : Aucun fichier dans la requête.")
        abort(400, "No file part in the request")

    file = request.files['file']

    if file.filename == "":
        print("Erreur : Aucun fichier sélectionné.")
        abort(400, "No file selected")

    # Utilise secure_filename pour éviter les problèmes liés aux noms de fichiers
    original_filename = secure_filename(file.filename)
    print("Nom de fichier original :", original_filename)

    # Génère un nom de fichier unique pour l'input
    input_filename = f"{uuid.uuid4().hex}_{original_filename}"
    input_path = os.path.join(UPLOAD_FOLDER, input_filename)
    print("Enregistrement du fichier uploadé dans :", input_path)

    try:
        file.save(input_path)
        print("Fichier sauvegardé avec succès.")
    except Exception as e:
        print("Erreur lors de la sauvegarde du fichier :", e)
        abort(500, "Erreur lors de l'écriture du fichier")

    # Prépare le chemin de sortie en ajoutant l'extension .packed
    output_filename = input_filename + ".packed"
    output_path = os.path.join(UPLOAD_FOLDER, output_filename)
    print("Le fichier packé sera généré sous :", output_path)

    # Chemin vers l'exécutable rPack. Adapte-le selon ton environnement.
    rpack_path = "../../rpack/target/debug/rpack"
    print("Tentative d'exécution de rPack :")
    print("Commande :", rpack_path, input_path, output_path)

    # Exécute rPack en passant l'input et l'output comme arguments.
    try:
        result = subprocess.run(
            [rpack_path, input_path, output_path],
            capture_output=True,
            text=True
        )
    except Exception as e:
        print("Exception lors de l'exécution de rPack :", e)
        abort(500, "Erreur lors de l'exécution de rPack")

    print("Code de retour de rPack :", result.returncode)
    print("Sortie standard de rPack :", result.stdout)
    print("Erreur standard de rPack :", result.stderr)

    if result.returncode != 0:
        print("Erreur : l'exécution de rPack a échoué.")
        abort(500, "rPack execution failed")

    # Vérifie que le fichier packé a bien été créé
    if not os.path.exists(output_path):
        print("Erreur : Le fichier packé n'a pas été généré.")
        abort(500, "Output file not generated")

    print("Fichier packé généré avec succès :", output_path)
    print("Envoi du fichier pour téléchargement...")

    # Renvoie le fichier généré au client avec un nom indiquant l'extension .packed
    download_filename = original_filename + ".packed"
    retour = send_file(output_path, as_attachment=True, download_name=download_filename)
    print(retour)

    os.remove(input_path)
    os.remove(output_path)
    print("Artifacts removed.\n===================================")

    return retour

if __name__ == '__main__':
    print("Démarrage du backend Flask sur le port 8080...")
    app.run(host="0.0.0.0", port=8080, debug=True)
