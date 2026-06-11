"""Simple Flask application for Docker learning."""
from flask import Flask, jsonify

app = Flask(__name__)


@app.route('/')
def hello():
    """Root endpoint."""
    return jsonify({
        'message': 'Hello from Docker!',
        'status': 'ok'
    })


@app.route('/health')
def health():
    """Health check endpoint."""
    return jsonify({'status': 'ok'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
