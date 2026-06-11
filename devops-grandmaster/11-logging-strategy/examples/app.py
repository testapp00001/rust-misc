"""Flask app with structured JSON logging."""
from flask import Flask, request, jsonify
import logging
import json
import sys
import uuid
from datetime import datetime


class JSONFormatter(logging.Formatter):
    """Format log records as JSON."""

    def format(self, record):
        log_entry = {
            'timestamp': datetime.utcnow().isoformat(),
            'level': record.levelname,
            'message': record.getMessage(),
            'module': record.module,
            'function': record.funcName,
            'line': record.lineno,
        }
        if hasattr(record, 'extra_data'):
            log_entry.update(record.extra_data)
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
        return json.dumps(log_entry)


# Set up structured logging
logger = logging.getLogger('app')
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger.addHandler(handler)
logger.setLevel(logging.INFO)

app = Flask(__name__)


@app.before_request
def before_request():
    request.request_id = str(uuid.uuid4())
    logger.info('Request started', extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'ip': request.remote_addr,
    }})


@app.after_request
def after_request(response):
    logger.info('Request completed', extra={'extra_data': {
        'request_id': request.request_id,
        'status_code': response.status_code,
    }})
    return response


@app.route('/')
def index():
    logger.info('Home page accessed', extra={'extra_data': {
        'request_id': request.request_id,
    }})
    return jsonify({'status': 'ok', 'message': 'Hello from structured logging!'})


@app.route('/error')
def error():
    try:
        raise ValueError('Something broke!')
    except Exception:
        logger.error('Error occurred', exc_info=True, extra={'extra_data': {
            'request_id': request.request_id,
        }})
        return jsonify({'error': 'Internal error'}), 500


@app.route('/health')
def health():
    return jsonify({'status': 'ok'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
