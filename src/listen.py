import os.path
from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError

SCOPES = ["https://www.googleapis.com/auth/gmail.readonly"]

# docs: https://developers.google.com/gmail/api/guides
# and: https://developers.google.com/gmail/imap/imap-smtp
# Following is a list of common terms used in the Gmail API:
#
# Message
# An email message containing the sender, recipients, subject, and body. After a message has been created, a message cannot be changed. A message is represented by a message resource.
# Thread
# A collection of related messages forming a conversation. In an email client app, a thread is formed when one or more recipients respond to a message with their own message.
# Label
# A mechanism for organizing messages and threads. For example, the label "taxes" might be created and applied to all messages and threads having to do with a user's taxes. There are two types of labels:
#
# System labels
# Internally-created labels, such as INBOX, TRASH, or SPAM. These labels cannot be deleted or modified. However, some system labels, such as INBOX can be applied to, or removed from, messages and threads.
# User labels
# Labels created by a user. These labels can be deleted or modified by the user or an application. A user label is represented by a label resource.
# Draft
# An unsent message. A message contained within the draft can be replaced. Sending a draft automatically deletes the draft and creates a message with the SENT system label. A draft is represented by a draft resource.


def get_service():
    """Shows basic usage of the Gmail API.
    Lists the user's Gmail labels.
    """
    creds = None
    tokens_file = "/home/v/tmp/token.json"
    credentials_file = "/home/v/tmp/credentials.json"
    # The file token.json stores the user's access and refresh tokens, and is
    # created automatically when the authorization flow completes for the first
    # time.
    if os.path.exists(tokens_file):
        creds = Credentials.from_authorized_user_file(tokens_file, SCOPES)
    # If there are no (valid) credentials available, let the user log in.
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            flow = InstalledAppFlow.from_client_secrets_file(credentials_file, SCOPES)
            creds = flow.run_local_server(port=0)
        # Save the credentials for the next run
        with open(tokens_file, "w") as token:
            token.write(creds.to_json())

    try:
        service = build("gmail", "v1", credentials=creds)
        return service
    except HttpError as error:
        print(f"An error occurred: {error}")


def check_new_messages(service):
    # Call the Gmail API to fetch INBOX
    results = (
        service.users()
        .messages()
        .list(userId="me", labelIds=["INBOX"], q="is:unread")
        .execute()
    )
    messages = results.get("messages", [])

    if not messages:
        print("No new messages.")
    else:
        print("New messages received:")
        for message in messages:
            msg = (
                service.users().messages().get(userId="me", id=message["id"]).execute()
            )
            print(f"Message snippet: {msg['snippet']}")


def main():
    service = get_service()
    check_new_messages(service)


if __name__ == "__main__":
    main()
